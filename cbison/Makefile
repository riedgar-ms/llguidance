TARGET = ../target/release

all:
	cd ../parser && cargo build --release
	# force static linking
	rm -f $(TARGET)/libllguidance.so $(TARGET)/libllguidance.dylib $(TARGET)/libllguidance.dll
	c++ -W -Wall -std=c++20 -o $(TARGET)/cbison test_cbison.cpp -I../parser -L$(TARGET) -lllguidance
	$(TARGET)/cbison

