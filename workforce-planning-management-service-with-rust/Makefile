.PHONY: demo

demo:
	cargo run -- db migrate
	cargo run -- task seed
	WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH=0 WORKFORCE_PLANNING_MANAGEMENT_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
