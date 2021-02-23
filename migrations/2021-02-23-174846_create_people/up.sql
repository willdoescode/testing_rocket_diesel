create table people (
    id serial primary key,
    first_name varchar not null,
    last_name varchar not null,
    age int not null,
    profession varchar not null,
    salary int not null
)