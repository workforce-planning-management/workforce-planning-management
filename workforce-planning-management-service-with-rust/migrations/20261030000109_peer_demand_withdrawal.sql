-- Migration: **a peer can withdraw a demand figure, and an older event does not overwrite a newer one** (WPM-R215, WPM-D146).
--
-- `programme_demands.peer_sent_at` is when the peer says it sent the last event applied to a received claim (it may not say; then
-- null), so an event sent earlier is ignored. `peer_withdrawn_at` is when the peer withdrew the claim by sending zero; such a claim
-- is `closed`, and its last figure is kept (the figure column still refuses zero). Both are null on a claim entered here.

ALTER TABLE programme_demands ADD COLUMN IF NOT EXISTS peer_sent_at TIMESTAMPTZ NULL;

ALTER TABLE programme_demands ADD COLUMN IF NOT EXISTS peer_withdrawn_at TIMESTAMPTZ NULL
                 CHECK (peer_withdrawn_at IS NULL OR (source = 'ppm' AND status = 'closed'));
