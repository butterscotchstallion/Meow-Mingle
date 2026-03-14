create table public.photos
(
    id         uuid                     default gen_random_uuid() not null
        constraint photos_pk
            primary key,
    "order"    integer                  default 0,
    created_at timestamp with time zone default now()
);

alter table public.photos
    owner to meowmingler;