test:
	docker run --rm \
		-v /sys/class/powercap:/sys/class/powercap \
		-v $(shell pwd):/code \
		-w /code \
		rust:bullseye \
		cargo test
