.PHONY: preview validate verify verify-browser serve wasm package

preview:
	./scripts/generate-preview.sh

validate:
	python3 scripts/validate_artifact.py --approved-profile

verify:
	./scripts/verify.sh

verify-browser:
	node scripts/verify-browser.mjs

serve:
	./scripts/serve.sh

wasm:
	./scripts/build-wasm.sh

package:
	./scripts/package.sh
