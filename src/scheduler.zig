//! KOGI Process Scheduler, Dispatcher & Queuing System
//! Provides task scheduling, priority-based dispatch, and work queues.

const std = @import("std");
const time_module = @import("time.zig");

/// Task priority levels
pub const TaskPriority = enum(u4) {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
};

/// Task states
pub const TaskState = enum {
    Pending,
    Ready,
    Running,
    Blocked,
    Completed,
    Failed,
};

/// Represents a schedulable task
pub const Task = struct {
    id: u64,
    priority: TaskPriority,
    state: TaskState,
    created_at: time_module.TimePoint,
    started_at: ?time_module.TimePoint = null,
    completed_at: ?time_module.TimePoint = null,
    deadline: ?time_module.TimePoint = null,
    execution_fn: *const fn (*anyopaque) void,
    context: *anyopaque,
    name: []const u8,
    allocator: std.mem.Allocator,

    pub fn execute(self: *Task) void {
        self.state = .Running;
        self.started_at = time_module.TimePoint.now();
        self.execution_fn(self.context);
        self.state = .Completed;
        self.completed_at = time_module.TimePoint.now();
    }

    pub fn executionTime(self: Task) ?time_module.Duration {
        if (self.started_at) |start| {
            if (self.completed_at) |end| {
                return start.duration(end);
            }
        }
        return null;
    }

    pub fn isOverdue(self: Task) bool {
        if (self.deadline) |deadline| {
            const now = time_module.TimePoint.now();
            const cmp = now.toNanoseconds() - deadline.toNanoseconds();
            return cmp > 0;
        }
        return false;
    }

    pub fn deinit(self: *Task) void {
        self.allocator.free(self.name);
    }
};

/// Task queue for FIFO processing
pub const TaskQueue = struct {
    queue: std.ArrayList(Task),
    allocator: std.mem.Allocator,
    mutex: std.Thread.Mutex = .{},
    not_empty: std.Thread.Condition = .{},

    pub fn init(allocator: std.mem.Allocator) TaskQueue {
        return TaskQueue{
            .queue = std.ArrayList(Task).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *TaskQueue) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.queue.items) |*task| {
            task.deinit();
        }
        self.queue.deinit();
    }

    pub fn enqueue(self: *TaskQueue, task: Task) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        try self.queue.append(task);
        self.not_empty.signal();
    }

    pub fn dequeue(self: *TaskQueue) ?Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.queue.items.len > 0) {
            return self.queue.orderedRemove(0);
        }
        return null;
    }

    pub fn waitForTask(self: *TaskQueue) Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        while (self.queue.items.len == 0) {
            self.not_empty.wait(&self.mutex);
        }

        return self.queue.orderedRemove(0);
    }

    pub fn length(self: *TaskQueue) usize {
        self.mutex.lock();
        defer self.mutex.unlock();
        return self.queue.items.len;
    }

    pub fn isEmpty(self: *TaskQueue) bool {
        return self.length() == 0;
    }
};

/// Priority queue for scheduling
pub const PriorityQueue = struct {
    heap: std.ArrayList(Task),
    allocator: std.mem.Allocator,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) PriorityQueue {
        return PriorityQueue{
            .heap = std.ArrayList(Task).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *PriorityQueue) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.heap.items) |*task| {
            task.deinit();
        }
        self.heap.deinit();
    }

    pub fn enqueue(self: *PriorityQueue, task: Task) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        try self.heap.append(task);
        self.bubbleUp(self.heap.items.len - 1);
    }

    pub fn dequeue(self: *PriorityQueue) ?Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.heap.items.len == 0) return null;

        const result = self.heap.items[0];
        if (self.heap.items.len > 1) {
            self.heap.items[0] = self.heap.pop();
            self.bubbleDown(0);
        } else {
            _ = self.heap.pop();
        }

        return result;
    }

    pub fn peek(self: *PriorityQueue) ?Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.heap.items.len > 0) {
            return self.heap.items[0];
        }
        return null;
    }

    pub fn length(self: *PriorityQueue) usize {
        self.mutex.lock();
        defer self.mutex.unlock();
        return self.heap.items.len;
    }

    fn bubbleUp(self: *PriorityQueue, idx: usize) void {
        if (idx == 0) return;
        const parent_idx = (idx - 1) / 2;

        if (@intFromEnum(self.heap.items[idx].priority) > @intFromEnum(self.heap.items[parent_idx].priority)) {
            std.mem.swap(Task, &self.heap.items[idx], &self.heap.items[parent_idx]);
            self.bubbleUp(parent_idx);
        }
    }

    fn bubbleDown(self: *PriorityQueue, idx: usize) void {
        const left_child = 2 * idx + 1;
        const right_child = 2 * idx + 2;
        var largest = idx;

        if (left_child < self.heap.items.len and
            @intFromEnum(self.heap.items[left_child].priority) > @intFromEnum(self.heap.items[largest].priority))
        {
            largest = left_child;
        }

        if (right_child < self.heap.items.len and
            @intFromEnum(self.heap.items[right_child].priority) > @intFromEnum(self.heap.items[largest].priority))
        {
            largest = right_child;
        }

        if (largest != idx) {
            std.mem.swap(Task, &self.heap.items[idx], &self.heap.items[largest]);
            self.bubbleDown(largest);
        }
    }
};

/// Delayed task queue for scheduled execution
pub const DelayedQueue = struct {
    tasks: std.ArrayList(Task),
    allocator: std.mem.Allocator,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) DelayedQueue {
        return DelayedQueue{
            .tasks = std.ArrayList(Task).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *DelayedQueue) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.tasks.items) |*task| {
            task.deinit();
        }
        self.tasks.deinit();
    }

    pub fn scheduleAfter(self: *DelayedQueue, task: Task, delay: time_module.Duration) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        const now = time_module.TimePoint.now();
        const scheduled_time = now.toNanoseconds() + delay.nanos;

        var modified_task = task;
        modified_task.deadline = time_module.TimePoint.fromNanoseconds(scheduled_time);

        try self.tasks.append(modified_task);
        // Keep sorted by deadline
        std.sort.insertion(Task, self.tasks.items, {}, compareByDeadline);
    }

    pub fn scheduleBefore(self: *DelayedQueue, task: Task, deadline: time_module.TimePoint) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        var modified_task = task;
        modified_task.deadline = deadline;

        try self.tasks.append(modified_task);
        std.sort.insertion(Task, self.tasks.items, {}, compareByDeadline);
    }

    pub fn readyTasks(self: *DelayedQueue) !std.ArrayList(Task) {
        self.mutex.lock();
        defer self.mutex.unlock();

        var ready = std.ArrayList(Task).init(self.allocator);
        const now = time_module.TimePoint.now();

        var i: usize = 0;
        while (i < self.tasks.items.len) {
            const task = self.tasks.items[i];
            if (task.deadline) |deadline| {
                if (now.toNanoseconds() >= deadline.toNanoseconds()) {
                    try ready.append(task);
                    _ = self.tasks.orderedRemove(i);
                    continue;
                }
            }
            i += 1;
        }

        return ready;
    }

    fn compareByDeadline(_: void, a: Task, b: Task) bool {
        const a_deadline = a.deadline orelse time_module.TimePoint{ .seconds = 0, .nanoseconds = 0 };
        const b_deadline = b.deadline orelse time_module.TimePoint{ .seconds = 0, .nanoseconds = 0 };
        return a_deadline.toNanoseconds() < b_deadline.toNanoseconds();
    }
};

/// Scheduler coordinates task execution
pub const Scheduler = struct {
    allocator: std.mem.Allocator,
    next_task_id: u64 = 1,
    ready_queue: PriorityQueue,
    delayed_queue: DelayedQueue,
    completed_tasks: std.ArrayList(Task),
    failed_tasks: std.ArrayList(Task),
    mutex: std.Thread.Mutex = .{},
    running: bool = false,

    pub fn init(allocator: std.mem.Allocator) Scheduler {
        return Scheduler{
            .allocator = allocator,
            .ready_queue = PriorityQueue.init(allocator),
            .delayed_queue = DelayedQueue.init(allocator),
            .completed_tasks = std.ArrayList(Task).init(allocator),
            .failed_tasks = std.ArrayList(Task).init(allocator),
        };
    }

    pub fn deinit(self: *Scheduler) void {
        self.ready_queue.deinit();
        self.delayed_queue.deinit();

        for (self.completed_tasks.items) |*task| {
            task.deinit();
        }
        self.completed_tasks.deinit();

        for (self.failed_tasks.items) |*task| {
            task.deinit();
        }
        self.failed_tasks.deinit();
    }

    pub fn createTask(
        self: *Scheduler,
        name: []const u8,
        priority: TaskPriority,
        execution_fn: *const fn (*anyopaque) void,
        context: *anyopaque,
    ) !Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        const task_id = self.next_task_id;
        self.next_task_id += 1;

        return Task{
            .id = task_id,
            .priority = priority,
            .state = .Pending,
            .created_at = time_module.TimePoint.now(),
            .execution_fn = execution_fn,
            .context = context,
            .name = try self.allocator.dupe(u8, name),
            .allocator = self.allocator,
        };
    }

    pub fn submitTask(self: *Scheduler, task: Task) !void {
        var t = task;
        t.state = .Ready;
        try self.ready_queue.enqueue(t);
    }

    pub fn scheduleTaskAfter(self: *Scheduler, task: Task, delay: time_module.Duration) !void {
        var t = task;
        t.state = .Pending;
        try self.delayed_queue.scheduleAfter(t, delay);
    }

    pub fn scheduleTaskAt(self: *Scheduler, task: Task, when: time_module.TimePoint) !void {
        var t = task;
        t.state = .Pending;
        try self.delayed_queue.scheduleBefore(t, when);
    }

    pub fn processPending(self: *Scheduler) !void {
        const ready = try self.delayed_queue.readyTasks();
        defer ready.deinit();

        for (ready.items) |t| {
            try self.submitTask(t);
        }
    }

    pub fn nextTask(self: *Scheduler) ?Task {
        return self.ready_queue.dequeue();
    }

    pub fn peekNextTask(self: *Scheduler) ?Task {
        return self.ready_queue.peek();
    }

    pub fn recordCompletion(self: *Scheduler, task: Task) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        try self.completed_tasks.append(task);
    }

    pub fn recordFailure(self: *Scheduler, task: Task) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        try self.failed_tasks.append(task);
    }

    pub fn stats(self: *Scheduler) struct { ready: usize, pending: usize, completed: usize, failed: usize } {
        self.mutex.lock();
        defer self.mutex.unlock();

        return .{
            .ready = self.ready_queue.length(),
            .pending = self.delayed_queue.tasks.items.len,
            .completed = self.completed_tasks.items.len,
            .failed = self.failed_tasks.items.len,
        };
    }
};

/// Dispatcher executes tasks from a scheduler
pub const Dispatcher = struct {
    allocator: std.mem.Allocator,
    scheduler: *Scheduler,
    thread_count: usize,
    threads: std.ArrayList(std.Thread),
    running: bool = false,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator, scheduler: *Scheduler, thread_count: usize) Dispatcher {
        return Dispatcher{
            .allocator = allocator,
            .scheduler = scheduler,
            .thread_count = thread_count,
            .threads = std.ArrayList(std.Thread).init(allocator),
        };
    }

    pub fn deinit(self: *Dispatcher) void {
        for (self.threads.items) |thread| {
            thread.join();
        }
        self.threads.deinit();
    }

    pub fn start(self: *Dispatcher) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.running) return;
        self.running = true;

        for (0..self.thread_count) |_| {
            const thread = try std.Thread.spawn(.{}, workerThread, .{self});
            try self.threads.append(thread);
        }
    }

    pub fn stop(self: *Dispatcher) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        self.running = false;
    }

    pub fn isRunning(self: *Dispatcher) bool {
        self.mutex.lock();
        defer self.mutex.unlock();
        return self.running;
    }
};

fn workerThread(dispatcher: *Dispatcher) void {
    while (dispatcher.isRunning()) {
        if (dispatcher.scheduler.nextTask()) |task| {
            var t = task;
            t.execute();
            _ = dispatcher.scheduler.recordCompletion(t) catch |err| {
                std.debug.print("Error recording task completion: {}\n", .{err});
            };
        } else {
            std.time.sleep(1_000_000); // Sleep 1ms if no tasks available
        }
    }
}

/// Work stealing queue for load balancing
pub const WorkStealingQueue = struct {
    queues: std.ArrayList(TaskQueue),
    allocator: std.mem.Allocator,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator, worker_count: usize) !WorkStealingQueue {
        var queues = std.ArrayList(TaskQueue).init(allocator);

        for (0..worker_count) |_| {
            try queues.append(TaskQueue.init(allocator));
        }

        return WorkStealingQueue{
            .queues = queues,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *WorkStealingQueue) void {
        for (self.queues.items) |*q| {
            q.deinit();
        }
        self.queues.deinit();
    }

    pub fn submitToQueue(self: *WorkStealingQueue, queue_idx: usize, task: Task) !void {
        if (queue_idx < self.queues.items.len) {
            try self.queues.items[queue_idx].enqueue(task);
        }
    }

    pub fn stealWork(self: *WorkStealingQueue, worker_idx: usize) ?Task {
        self.mutex.lock();
        defer self.mutex.unlock();

        // Try to steal from other queues
        for (0..self.queues.items.len) |i| {
            if (i != worker_idx) {
                if (self.queues.items[i].dequeue()) |task| {
                    return task;
                }
            }
        }

        return null;
    }
};
