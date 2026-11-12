.PHONY: demo test test-github-discussions

# Start both service and UI for the demo
# Note: The service runs on port 1241, the UI connects to it
demo:
	@echo "Starting WPM service in background..."
	cd workforce-planning-management-service-with-rust && make demo &
	sleep 5
	@echo "Starting WPM UI..."
	cd workforce-planning-management-ui-with-svelte && make demo

# Run GitHub Discussions tests
test-github-discussions:
	@echo "Running GitHub Discussions tests..."
	./bin/test-github-discussions

# Run all tests
test: test-github-discussions
