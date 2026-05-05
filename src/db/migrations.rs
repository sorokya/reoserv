use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};

use super::{DbHandle, insert_params};

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS `migrations` (
    `migration_name` VARCHAR(255) PRIMARY KEY,
    `applied_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Driver {
    Mysql,
    Sqlite,
}

impl TryFrom<&str> for Driver {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "mysql" => Ok(Self::Mysql),
            "sqlite" => Ok(Self::Sqlite),
            other => Err(anyhow!("Unsupported database driver: {}", other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MigrationKind {
    Generic,
    Mysql,
    Sqlite,
}

#[derive(Debug, Default)]
struct MigrationFiles {
    generic: Option<MigrationFile>,
    mysql: Option<MigrationFile>,
    sqlite: Option<MigrationFile>,
}

#[derive(Clone, Debug)]
struct MigrationFile {
    logical_name: String,
    path: PathBuf,
    script: String,
    directives: Vec<Directive>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Directive {
    SkipIfTableExists(String),
    SkipIfTableMissing(String),
}

pub async fn run_startup_migrations(db: &DbHandle, driver: &str) -> anyhow::Result<()> {
    run_startup_migrations_in_dir(db, driver, Path::new("data/migrations")).await
}

async fn run_startup_migrations_in_dir(
    db: &DbHandle,
    driver: &str,
    dir: &Path,
) -> anyhow::Result<()> {
    let driver = Driver::try_from(driver)?;

    ensure_migrations_table(db).await?;
    let applied_migrations = load_applied_migrations(db).await?;
    let migrations = discover_migrations(dir, driver)?;

    for migration in migrations {
        if applied_migrations.contains(&migration.logical_name) {
            continue;
        }

        if !should_skip_migration(db, driver, &migration).await? {
            tracing::info!(
                "Applying migration {} from {}",
                migration.logical_name,
                migration.path.display()
            );
            db.execute(&migration.script).await.with_context(|| {
                format!(
                    "Failed to execute migration {} ({})",
                    migration.logical_name,
                    migration.path.display()
                )
            })?;
        }

        record_migration_if_missing(db, &migration.logical_name).await?;
    }

    Ok(())
}

async fn ensure_migrations_table(db: &DbHandle) -> anyhow::Result<()> {
    db.execute(MIGRATIONS_TABLE_SQL).await
}

async fn load_applied_migrations(db: &DbHandle) -> anyhow::Result<HashSet<String>> {
    let rows = db
        .query("SELECT `migration_name` FROM `migrations`")
        .await
        .context("Failed to read applied migrations")?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.get_string(0))
        .collect::<HashSet<_>>())
}

fn discover_migrations(dir: &Path, driver: Driver) -> anyhow::Result<Vec<MigrationFile>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read migrations dir {}", dir.display()))?;
    let mut grouped_files = BTreeMap::<String, MigrationFiles>::new();

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((logical_name, kind)) = parse_migration_file_name(file_name) else {
            continue;
        };

        let script = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read migration {}", path.display()))?;
        let migration = MigrationFile {
            logical_name: logical_name.clone(),
            directives: parse_directives(&script),
            path,
            script,
        };

        let files = grouped_files.entry(logical_name).or_default();
        match kind {
            MigrationKind::Generic => files.generic = Some(migration),
            MigrationKind::Mysql => files.mysql = Some(migration),
            MigrationKind::Sqlite => files.sqlite = Some(migration),
        }
    }

    let migrations = grouped_files
        .into_values()
        .filter_map(|files| select_migration_file(files, driver))
        .collect();

    Ok(migrations)
}

fn parse_migration_file_name(file_name: &str) -> Option<(String, MigrationKind)> {
    if let Some(logical_name) = file_name.strip_suffix(".mysql.sql") {
        return Some((logical_name.to_string(), MigrationKind::Mysql));
    }

    if let Some(logical_name) = file_name.strip_suffix(".sqlite.sql") {
        return Some((logical_name.to_string(), MigrationKind::Sqlite));
    }

    file_name
        .strip_suffix(".sql")
        .map(|logical_name| (logical_name.to_string(), MigrationKind::Generic))
}

fn select_migration_file(files: MigrationFiles, driver: Driver) -> Option<MigrationFile> {
    match driver {
        Driver::Mysql => files.mysql.or(files.generic),
        Driver::Sqlite => files.sqlite.or(files.generic),
    }
}

fn parse_directives(script: &str) -> Vec<Directive> {
    let mut directives = Vec::new();

    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !trimmed.starts_with("--") {
            break;
        }

        let Some(directive) = trimmed.strip_prefix("-- reoserv:") else {
            continue;
        };

        let directive = directive.trim();
        if let Some(table_name) = directive.strip_prefix("skip-if-table-exists=") {
            directives.push(Directive::SkipIfTableExists(table_name.trim().to_string()));
        } else if let Some(table_name) = directive.strip_prefix("skip-if-table-missing=") {
            directives.push(Directive::SkipIfTableMissing(table_name.trim().to_string()));
        }
    }

    directives
}

async fn should_skip_migration(
    db: &DbHandle,
    driver: Driver,
    migration: &MigrationFile,
) -> anyhow::Result<bool> {
    for directive in &migration.directives {
        match directive {
            Directive::SkipIfTableExists(table_name) => {
                if table_exists(db, driver, table_name).await? {
                    return Ok(true);
                }
            }
            Directive::SkipIfTableMissing(table_name) => {
                if !table_exists(db, driver, table_name).await? {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

async fn table_exists(db: &DbHandle, driver: Driver, table_name: &str) -> anyhow::Result<bool> {
    let query = match driver {
        Driver::Mysql => insert_params(
            "SELECT COUNT(1) FROM `information_schema`.`tables` WHERE `table_schema` = DATABASE() AND `table_name` = :table_name",
            &[("table_name", &table_name)],
        ),
        Driver::Sqlite => insert_params(
            "SELECT COUNT(1) FROM `sqlite_master` WHERE `type` = 'table' AND `name` = :table_name",
            &[("table_name", &table_name)],
        ),
    };

    Ok(db.query_int(&query).await?.unwrap_or_default() > 0)
}

async fn record_migration_if_missing(db: &DbHandle, migration_name: &str) -> anyhow::Result<()> {
    let existing = db
        .query_int(&insert_params(
            "SELECT COUNT(1) FROM `migrations` WHERE `migration_name` = :migration_name",
            &[("migration_name", &migration_name)],
        ))
        .await?
        .unwrap_or_default();

    if existing == 0 {
        db.execute(&insert_params(
            "INSERT INTO `migrations` (`migration_name`) VALUES (:migration_name)",
            &[("migration_name", &migration_name)],
        ))
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::db::Connection;

    fn unique_temp_dir() -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("reoserv-migrations-{}-{}", timestamp, counter))
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[tokio::test]
    async fn prefers_driver_specific_migration_files() {
        let dir = unique_temp_dir();
        write_file(
            &dir.join("0001-bootstrap.sql"),
            "CREATE TABLE generic_table (id INTEGER PRIMARY KEY);",
        );
        write_file(
            &dir.join("0001-bootstrap.sqlite.sql"),
            "CREATE TABLE driver_selected_table (id INTEGER PRIMARY KEY);",
        );

        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let db = DbHandle::new(connection);

        run_startup_migrations_in_dir(&db, "sqlite", &dir)
            .await
            .unwrap();

        assert!(
            table_exists(&db, Driver::Sqlite, "driver_selected_table")
                .await
                .unwrap()
        );
        assert!(
            !table_exists(&db, Driver::Sqlite, "generic_table")
                .await
                .unwrap()
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn skips_migrations_based_on_table_directives() {
        let dir = unique_temp_dir();
        write_file(
            &dir.join("0001-bootstrap.sqlite.sql"),
            "-- reoserv: skip-if-table-exists=accounts\nCREATE TABLE `should_not_exist` (`id` INTEGER PRIMARY KEY);",
        );
        write_file(
            &dir.join("0002-legacy.sql"),
            "-- reoserv: skip-if-table-missing=Account\nCREATE TABLE `legacy_loaded` (`id` INTEGER PRIMARY KEY);",
        );

        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let db = DbHandle::new(connection);
        db.execute("CREATE TABLE `accounts` (`id` INTEGER PRIMARY KEY);")
            .await
            .unwrap();

        run_startup_migrations_in_dir(&db, "sqlite", &dir)
            .await
            .unwrap();

        let applied = db
            .query("SELECT `migration_name` FROM `migrations` ORDER BY `migration_name`")
            .await
            .unwrap();
        assert_eq!(applied.len(), 2);
        assert_eq!(applied[0].get_string(0).as_deref(), Some("0001-bootstrap"));
        assert_eq!(applied[1].get_string(0).as_deref(), Some("0002-legacy"));
        assert!(
            !table_exists(&db, Driver::Sqlite, "should_not_exist")
                .await
                .unwrap()
        );
        assert!(
            !table_exists(&db, Driver::Sqlite, "legacy_loaded")
                .await
                .unwrap()
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn runs_generic_migration_when_no_driver_specific_file_exists() {
        let dir = unique_temp_dir();
        write_file(
            &dir.join("0001-generic.sql"),
            "CREATE TABLE generic_only (id INTEGER PRIMARY KEY);",
        );

        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let db = DbHandle::new(connection);

        run_startup_migrations_in_dir(&db, "sqlite", &dir)
            .await
            .unwrap();

        assert!(
            table_exists(&db, Driver::Sqlite, "generic_only")
                .await
                .unwrap()
        );

        fs::remove_dir_all(dir).unwrap();
    }
}
