create table cats_photos
(
    cat_id   uuid not null
        constraint cats_photos___fk_cat_id
            references cats,
    photo_id uuid not null
        constraint cats_photos___fk_photo_id
            references photos (id),
    constraint cats_photos_pk_cat_photo_id_unique
        unique (cat_id, photo_id)
);

alter table cats_photos
    owner to meowmingler;

pg_dump -t 'postgres.photos' --schema-only database-name