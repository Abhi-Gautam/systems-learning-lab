CXX ?= clang++
CXXFLAGS ?= -std=c++17 -Wall -Wextra -Werror=return-type -O0 -g
ASAN_FLAGS = -fsanitize=address -fno-omit-frame-pointer
COMMON = common/test_util.h
PROBLEMS = 00-singly-list 01-doubly-list 02-lc707-design-linked-list 03-lc622-circular-queue 04-lc641-circular-deque 05-lc1670-front-middle-back-queue 06-lc232-queue-using-stacks 07-lc225-stack-using-queues 08-lc155-min-stack 09-lc1381-stack-with-increment 10-lc705-hash-set 11-lc706-hash-map 12-lc380-insert-delete-getrandom 13-open-address-map 14-lc146-lru-cache 15-lc460-lfu-cache 16-lc432-all-oone 17-lc895-max-freq-stack 18-lc208-trie 19-lc211-word-dictionary 20-bst 21-lc173-bst-iterator 22-lc1206-skiplist 23-free-list-allocator 24-intrusive-list 25-binary-heap 26-lc1472-browser-history

.PHONY: help all compile-all test-all clean 00-singly-list 01-doubly-list 02-lc707-design-linked-list 03-lc622-circular-queue 04-lc641-circular-deque 05-lc1670-front-middle-back-queue 06-lc232-queue-using-stacks 07-lc225-stack-using-queues 08-lc155-min-stack 09-lc1381-stack-with-increment 10-lc705-hash-set 11-lc706-hash-map 12-lc380-insert-delete-getrandom 13-open-address-map 14-lc146-lru-cache 15-lc460-lfu-cache 16-lc432-all-oone 17-lc895-max-freq-stack 18-lc208-trie 19-lc211-word-dictionary 20-bst 21-lc173-bst-iterator 22-lc1206-skiplist 23-free-list-allocator 24-intrusive-list 25-binary-heap 26-lc1472-browser-history test-00-singly-list test-01-doubly-list test-02-lc707-design-linked-list test-03-lc622-circular-queue test-04-lc641-circular-deque test-05-lc1670-front-middle-back-queue test-06-lc232-queue-using-stacks test-07-lc225-stack-using-queues test-08-lc155-min-stack test-09-lc1381-stack-with-increment test-10-lc705-hash-set test-11-lc706-hash-map test-12-lc380-insert-delete-getrandom test-13-open-address-map test-14-lc146-lru-cache test-15-lc460-lfu-cache test-16-lc432-all-oone test-17-lc895-max-freq-stack test-18-lc208-trie test-19-lc211-word-dictionary test-20-bst test-21-lc173-bst-iterator test-22-lc1206-skiplist test-23-free-list-allocator test-24-intrusive-list test-25-binary-heap test-26-lc1472-browser-history
help:
	@printf "Targets:\n  make <problem>          build one\n  make test-<problem>     build and run tests\n  make compile-all        compile every scaffold\n  make test-all           run every test (stubs are expected to fail)\n  make clean              remove build artifacts\n"

all: compile-all
compile-all: $(PROBLEMS)
test-all: test-00-singly-list test-01-doubly-list test-02-lc707-design-linked-list test-03-lc622-circular-queue test-04-lc641-circular-deque test-05-lc1670-front-middle-back-queue test-06-lc232-queue-using-stacks test-07-lc225-stack-using-queues test-08-lc155-min-stack test-09-lc1381-stack-with-increment test-10-lc705-hash-set test-11-lc706-hash-map test-12-lc380-insert-delete-getrandom test-13-open-address-map test-14-lc146-lru-cache test-15-lc460-lfu-cache test-16-lc432-all-oone test-17-lc895-max-freq-stack test-18-lc208-trie test-19-lc211-word-dictionary test-20-bst test-21-lc173-bst-iterator test-22-lc1206-skiplist test-23-free-list-allocator test-24-intrusive-list test-25-binary-heap test-26-lc1472-browser-history

define BUILD_template
$(1):
	@mkdir -p build bin
	$(CXX) $(CXXFLAGS) $(if $(ASAN),$(ASAN_FLAGS),) $(1)/impl.cpp $(1)/test.cpp -I$(1) -Icommon -o bin/$(1)
test-$(1): $(1)
	@bin/$(1)
endef

$(eval $(call BUILD_template,00-singly-list))
$(eval $(call BUILD_template,01-doubly-list))
$(eval $(call BUILD_template,02-lc707-design-linked-list))
$(eval $(call BUILD_template,03-lc622-circular-queue))
$(eval $(call BUILD_template,04-lc641-circular-deque))
$(eval $(call BUILD_template,05-lc1670-front-middle-back-queue))
$(eval $(call BUILD_template,06-lc232-queue-using-stacks))
$(eval $(call BUILD_template,07-lc225-stack-using-queues))
$(eval $(call BUILD_template,08-lc155-min-stack))
$(eval $(call BUILD_template,09-lc1381-stack-with-increment))
$(eval $(call BUILD_template,10-lc705-hash-set))
$(eval $(call BUILD_template,11-lc706-hash-map))
$(eval $(call BUILD_template,12-lc380-insert-delete-getrandom))
$(eval $(call BUILD_template,13-open-address-map))
$(eval $(call BUILD_template,14-lc146-lru-cache))
$(eval $(call BUILD_template,15-lc460-lfu-cache))
$(eval $(call BUILD_template,16-lc432-all-oone))
$(eval $(call BUILD_template,17-lc895-max-freq-stack))
$(eval $(call BUILD_template,18-lc208-trie))
$(eval $(call BUILD_template,19-lc211-word-dictionary))
$(eval $(call BUILD_template,20-bst))
$(eval $(call BUILD_template,21-lc173-bst-iterator))
$(eval $(call BUILD_template,22-lc1206-skiplist))
$(eval $(call BUILD_template,23-free-list-allocator))
$(eval $(call BUILD_template,24-intrusive-list))
$(eval $(call BUILD_template,25-binary-heap))
$(eval $(call BUILD_template,26-lc1472-browser-history))

clean:
	rm -rf bin build
