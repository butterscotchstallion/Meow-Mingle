create table public.sessions
(
    id         uuid                     default gen_random_uuid(),
    created_at timestamp with time zone default now(),
    updated_at timestamp with time zone,
    active     boolean                  default true,
    cat_id     uuid                                               not null
        constraint sessions_pk_unique_cat_id
            unique
        constraint sessions___fk_cat_id
            references public.cats,
    session_id uuid                     default gen_random_uuid() not null
);

alter table public.sessions
    owner to meowmingler;

