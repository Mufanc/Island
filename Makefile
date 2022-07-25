.PHONY : clean run

SRCS	:= $(shell find src -name "*.rs")
TARGET	:= "target/release/island"

ifeq (run, $(firstword $(MAKECMDGOALS)))
  ARGS := $(wordlist 2, $(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(ARGS): ;@:)
endif

island: $(SRCS)
	cargo build --release

clean:
	rm -rf target/*

run: island
	sudo chown root $(TARGET)
	sudo chmod 4755 $(TARGET)
	$(TARGET) $(ARGS)

install: island
	sudo install -b -o root -m 4755 $(TARGET) /usr/bin/
