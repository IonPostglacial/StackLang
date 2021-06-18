all: interp

interp: ./src/main.c
	gcc $^ -o $@

clean:
	rm ./interp