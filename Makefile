.PHONY : clean run install

SRCS	:= $(shell find src -name "*.rs")
TARGET	:= "target/release/island"

ifeq (run, $(firstword $(MAKECMDGOALS)))
  ARGS := $(wordlist 2, $(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(ARGS): ;@:)
endif

island: $(SRCS)
	cargo build

clean:
	rm -rf target/*

run: island
	sudo chown root $(TARGET)
	sudo chmod 4755 $(TARGET)
	$(TARGET) $(ARGS)

install: $(SRCS)
	cargo build --release
	sudo install -b -o root -m 4755 $(TARGET) /usr/bin/
