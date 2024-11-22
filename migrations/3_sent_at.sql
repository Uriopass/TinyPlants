ALTER TABLE water DROP COLUMN notif_sent    integer default 0;
ALTER TABLE water ADD  COLUMN notif_sent_at integer default null;