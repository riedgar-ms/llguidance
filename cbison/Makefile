TARGET = ../target/release

all:
	cd ../parser && cargo build --release
	c++ -W -Wall -std=c++20 -o $(TARGET)/cbison test_cbison.cpp -I../parser -I../llguidance_cbison -I. -L$(TARGET) -lllguidance_cbison
	$(TARGET)/cbison

