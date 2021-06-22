all: interp

interp: ./src/main.c
	gcc -Wall --debug $^ -o $@

clean:
	rm ./interp