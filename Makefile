run:
	npx @tailwindcss/cli -i ./tailwind.css -o ./assets/tailwind.css --content './src/**/*.{rs,html}' & cargo run
