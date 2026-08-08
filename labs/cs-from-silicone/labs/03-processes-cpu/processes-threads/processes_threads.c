// ============================================================================
// processes_threads.c — Processes and threads, proven with printed IDs/addresses
// ============================================================================
//
// Build + run:
//   clang -std=c11 -O0 -g -Wall -Wextra -pthread processes_threads.c -o /tmp/processes_threads
//   /tmp/processes_threads
//
// Read this file top to bottom, run it, then explain the numbers it prints.
//
// What this first experiment proves:
//   * A process is the owner of one virtual-address-space mapping.
//   * A thread is one independently schedulable CPU context inside that process.
//   * Threads have distinct stack locals, but they see one global variable.
//   * fork() gives another process the same virtual-address numbers initially.
//     The child's first write triggers copy-on-write, so it becomes private.
//
// Do NOT infer that the final thread/parent values prove an unsynchronized data
// race is safe. pthread_join() is deliberately present: it supplies the handoff
// between worker completion and the main thread's final read. We will remove
// that protection when we reach atomics and memory ordering.

#include <inttypes.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#if defined(__APPLE__)
#include <pthread.h>
#elif defined(__linux__)
#include <sys/syscall.h>
#endif

// This is storage in the program's data mapping, not in either thread's stack.
// Both threads use the SAME process page tables, so &shared_counter prints as
// the same virtual address in main and worker, and worker's STORE changes what
// main later LOADs. It starts at 10 only to make each mutation obvious.
static int shared_counter = 10;

// pthread_t is an opaque library handle, not necessarily the kernel's numeric
// thread ID. Print the OS-visible ID instead: macOS exposes it through pthread;
// Linux exposes it through gettid(2). Later, `ps -M`/`/proc` can show the same
// distinction from outside the program.
static uint64_t current_thread_id(void) {
#if defined(__APPLE__)
    uint64_t id = 0;
    if (pthread_threadid_np(NULL, &id) != 0) {
        perror("pthread_threadid_np");
        exit(EXIT_FAILURE);
    }
    return id;
#elif defined(__linux__)
    return (uint64_t)syscall(SYS_gettid);
#else
#error "This lab currently supports macOS and Linux."
#endif
}

static void *worker(void *unused) {
    (void)unused;

    // `worker_local` is addressed relative to THIS thread's stack pointer. Its
    // address must differ from main_local even though PID and page tables match.
    int worker_local = 200;

    printf("worker: pid=%d tid=%" PRIu64 " &worker_local=%p &shared_counter=%p value=%d\n",
           getpid(), current_thread_id(), (void *)&worker_local,
           (void *)&shared_counter, shared_counter);

    // At -O0, conceptually: load address of shared_counter; store 42 there.
    // No data crosses a process boundary. This is an ordinary write through a
    // mapping both threads already own.
    shared_counter = 42;
    return NULL;
}

int main(void) {
    pthread_t thread;

    // This local is in main's call frame, so it belongs to the main thread's
    // execution context rather than to the process-wide data mapping.
    int main_local = 100;

    printf("main:   pid=%d tid=%" PRIu64 " &main_local=%p &shared_counter=%p value=%d\n",
           getpid(), current_thread_id(), (void *)&main_local,
           (void *)&shared_counter, shared_counter);

    if (pthread_create(&thread, NULL, worker, NULL) != 0) {
        perror("pthread_create");
        return EXIT_FAILURE;
    }
    if (pthread_join(thread, NULL) != 0) {
        perror("pthread_join");
        return EXIT_FAILURE;
    }

    // join waits for the worker to terminate and establishes the synchronization
    // needed for this read to observe the completed worker's store.
    printf("main:   after join, shared_counter=%d (worker's store is visible)\n",
           shared_counter);

    // We joined before fork(), leaving one thread. That avoids the hard rule for
    // fork in a multithreaded process: before exec(), the child must only call
    // async-signal-safe functions because library locks may be left held by
    // threads that no longer exist in the child.
    fflush(stdout);
    pid_t child = fork();
    if (child < 0) {
        perror("fork");
        return EXIT_FAILURE;
    }

    if (child == 0) {
        // Parent and child print the SAME virtual address. fork initially gives
        // the child page-table entries that refer to the same frames, marked
        // copy-on-write. It does not eagerly copy the process's entire memory.
        printf("child:  pid=%d tid=%" PRIu64 " &shared_counter=%p before=%d\n",
               getpid(), current_thread_id(), (void *)&shared_counter, shared_counter);

        // This write faults on the COW mapping. The kernel gives the child a
        // private writable physical frame, copies this page's old bytes, then
        // resumes this STORE. The parent therefore keeps its own value, 42.
        shared_counter = 99;
        printf("child:  wrote 99; its private post-COW value=%d\n", shared_counter);
        fflush(stdout);
        _exit(EXIT_SUCCESS);
    }

    if (waitpid(child, NULL, 0) < 0) {
        perror("waitpid");
        return EXIT_FAILURE;
    }

    // Same numeric virtual address as the child, different physical frame after
    // the child's write. This is the concrete correction to “same pointer value
    // means shared memory”: the process's page tables give the number meaning.
    printf("parent: pid=%d &shared_counter=%p after child exit=%d\n",
           getpid(), (void *)&shared_counter, shared_counter);
    puts("proof: threads share this mapping; fork shares lazily, then writes split it.");
    return EXIT_SUCCESS;
}
