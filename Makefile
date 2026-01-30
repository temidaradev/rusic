run:
	npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css --content './src/**/*.{rs,html}' & cargo run

flatpak-build:
	flatpak-builder --user --install --force-clean build-dir com.temidaradev.rusic.json

flatpak-run:
	flatpak run com.temidaradev.rusic
