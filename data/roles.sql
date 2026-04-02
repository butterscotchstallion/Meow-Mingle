create table roles
(
    id   uuid default gen_random_uuid() not null
        constraint roles_pk
            primary key,
    name varchar(50)                    not null,
    slug varchar(60)                    not null
);

alter table roles
    owner to meowmingler;

INSERT INTO public.roles (id, name, slug)
VALUES ('b6c4fa44-63ba-4319-af6f-d68be0e8f385', 'Cat Admin', 'cat-admin');
