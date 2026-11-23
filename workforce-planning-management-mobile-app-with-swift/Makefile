# Workforce Planning Management for iPhone and iPad (spec/mobile-apps.md, WPM-T638).
# Lily comes from a checkout of the Lily monorepo: set LILY_DESIGN_SYSTEM_DIR (WPM-D226).
SIMULATOR ?= generic/platform=iOS Simulator

.PHONY: build test app clean
build:
	swift build
test:
	swift test
app:
	xcodebuild -project App/WorkforcePlanningManagement.xcodeproj -scheme WorkforcePlanningManagement \
		-destination '$(SIMULATOR)' build
clean:
	rm -rf .build
