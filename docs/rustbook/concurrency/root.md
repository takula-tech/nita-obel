# `Concurrency`

Common concurrency approaches that include the following:

- A background thread that periodically wakes up to do its job

- Pipelines where data flows from one thread to the next  
  via lock-free channel, with each thread doing a little of the work

- ECS programming model that utilizes all the cpu cores  
  to process data in parallels

- Atomic Data types: using cas api of atomic integer to run some initialization logics only once

- Mutex to protect the concurrent access to shared data

- Task pool to run some tasks in parallel
