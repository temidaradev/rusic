serve:
	dx serve

tailwind:
	npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css --content './rusic/**/*.rs,./components/**/*.rs,./pages/**/*.rs,./hooks/**/*.rs,./player/**/*.rs,./reader/**/*.rs'

build: tailwind
	dx build --package rusic --release
	@echo ""
	@echo "Build complete!"

run-release:
	cd target/dx/rusic/release/linux/app && ./rusic

flatpak:
	@chmod +x build-flatpak.sh
	./build-flatpak.sh

flatpak-install: flatpak

flatpak-run:
	flatpak run com.temidaradev.rusic

clean:
	cargo clean
	rm -rf target/dx dist build-dir .flatpak-builder

