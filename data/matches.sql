create table public.matches
(
    id           uuid                     default gen_random_uuid() not null
        constraint matches_pk
            primary key,
    initiator_id uuid                                               not null
        constraint matches___fk_cat_id
            references public.cats,
    target_id    uuid                                               not null,
    created_at   timestamp with time zone default now(),
    status       match_status             default 'pending'::match_status,
    seen         boolean                  default false,
    constraint matches_pk_unique_init_target
        unique (initiator_id, target_id)
);

comment on column public.matches.initiator_id is 'initiator of match';

comment on column public.matches.target_id is 'the match chosen by initiator_id';

alter table public.matches
    owner to meowmingler;

