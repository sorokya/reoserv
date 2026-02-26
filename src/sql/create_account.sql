INSERT INTO
    `accounts` (
        `name`,
        `password_hash`,
        `real_name`,
        `location`,
        `email`,
        `computer`,
        `hdid`
    )
VALUES
    (
        :name,
        :password_hash,
        :real_name,
        :location,
        :email,
        :computer,
        :hdid
    );