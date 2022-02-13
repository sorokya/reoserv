INSERT INTO
    `reoserv`.`Account` (
        `name`,
        `password_hash`,
        `real_name`,
        `location`,
        `email`,
        `computer`,
        `hdid`,
        `register_ip`,
        `created_at`
    )
VALUES
    (
        :name,
        :password_hash,
        :real_name,
        :location,
        :email,
        :computer,
        :hdid,
        :register_ip,
        NOW()
    );