.PHONY: demo

demo:
	cargo run -- db migrate
	cargo run -- task seed
	WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
