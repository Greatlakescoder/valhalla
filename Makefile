go: build run
	
build:
	docker build . -t odin_scanner 
run:
	docker run --rm --network host odin_scanner
investigate:
	docker run -it --network host odin_scanner