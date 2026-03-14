create table public.cats
(
    id              uuid                     default gen_random_uuid() not null
        constraint cats_pk
            primary key,
    name            varchar(36)                                        not null
        constraint cats_unique_username
            unique,
    password        varchar(150)                                       not null,
    created_at      timestamp with time zone default now(),
    updated_at      timestamp with time zone,
    active          boolean                  default true,
    avatar_filename varchar(255),
    biography       varchar(500),
    breed_id        uuid                                               not null,
    birth_date      timestamp with time zone                           not null,
    last_seen       timestamp with time zone
);

alter table public.cats
    owner to meowmingler;

