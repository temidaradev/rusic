serve:
	dx serve

tailwind:
	npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css --content './rusic/**/*.rs,./components/**/*.rs,./pages/**/*.rs,./hooks/**/*.rs,./player/**/*.rs,./reader/**/*.rs'

build: tailwind
	dx build --package rusic --release
	@echo ""
	@echo "Build complete! Run 'make install' to install."

run-release:
	cd target/dx/rusic/release/linux/app && ./rusic

clean:
	cargo clean
	rm -rf target/dx dist
