serve:
	dx serve

tailwind:
	npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css --content './rusic/**/*.rs,./components/**/*.rs,./pages/**/*.rs,./hooks/**/*.rs,./player/**/*.rs,./reader/**/*.rs'

build: tailwind
	dx build --package rusic --release

run-release: build
	cd target/dx/rusic/release/linux/app && ./rusic

clean:
	cargo clean
	rm -rf target/dx dist
