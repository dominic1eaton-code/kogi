const std = @import("std");
const worker_module = @import("worker.zig");
const accounts_module = @import("accounts.zig");
const portfolio_module = @import("portfolio.zig");
const kernel_module = @import("kernel.zig");
const crm_module = @import("crm.zig");
const assets_module = @import("assets.zig");

// ============ Worker Module Tests ============

test "WorkerManager initialization" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    try std.testing.expectEqual(@as(usize, 0), manager.workers.items.len);
}

test "register worker" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    const worker_id = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.developer);
    try std.testing.expectEqual(@as(u32, 0), worker_id);
    try std.testing.expectEqual(@as(usize, 1), manager.workers.items.len);

    const worker = manager.workers.items[0];
    try std.testing.expectEqualStrings("Alice", worker.name);
    try std.testing.expectEqualStrings("alice@example.com", worker.email);
    try std.testing.expectEqual(@as(f32, 75.50), worker.hourly_rate);
    try std.testing.expect(worker.active);
    try std.testing.expectEqual(worker_module.WorkerType.developer, worker.worker_type);
}

test "register multiple workers" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    const id1 = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.freelancer);
    const id2 = try manager.registerWorker("Bob", "bob@example.com", 65.00, worker_module.WorkerType.contractor);
    const id3 = try manager.registerWorker("Carol", "carol@example.com", 85.75, worker_module.WorkerType.artist);

    try std.testing.expectEqual(@as(u32, 0), id1);
    try std.testing.expectEqual(@as(u32, 1), id2);
    try std.testing.expectEqual(@as(u32, 2), id3);
    try std.testing.expectEqual(@as(usize, 3), manager.workers.items.len);
}

test "add skill to worker" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    const worker_id = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.developer);
    try manager.addSkill(worker_id, "Zig Programming");
    try manager.addSkill(worker_id, "System Design");

    const worker = manager.workers.items[0];
    try std.testing.expectEqual(@as(usize, 2), worker.skills.items.len);
    try std.testing.expectEqualStrings("Zig Programming", worker.skills.items[0]);
    try std.testing.expectEqualStrings("System Design", worker.skills.items[1]);
}

test "get active workers count" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    try std.testing.expectEqual(@as(u32, 0), manager.getActiveCount());

    _ = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.gig);
    _ = try manager.registerWorker("Bob", "bob@example.com", 65.00, worker_module.WorkerType.consultant);

    try std.testing.expectEqual(@as(u32, 2), manager.getActiveCount());
}

test "filter workers by type" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    _ = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.freelancer);
    _ = try manager.registerWorker("Bob", "bob@example.com", 65.00, worker_module.WorkerType.freelancer);
    _ = try manager.registerWorker("Carol", "carol@example.com", 85.75, worker_module.WorkerType.artist);

    var results = try manager.getWorkersByType(worker_module.WorkerType.freelancer, allocator);
    defer results.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 2), results.items.len);
    try std.testing.expectEqualStrings("Alice", results.items[0].name);
    try std.testing.expectEqualStrings("Bob", results.items[1].name);
}

// ============ CRM Tests ============

test "crm manager basic client" {
    const allocator = std.testing.allocator;
    var crm = crm_module.CRMManager.init(allocator);
    defer crm.deinit();

    const cid = try crm.addClient("Acme", "Tech", "Important");
    try std.testing.expectEqual(@as(u32, 0), cid);
    try std.testing.expectEqual(@as(u32, 1), crm.getClientCount());

    const conid = try crm.addContactToClient(cid, "John", "john@acme.com", null, "CEO");
    try std.testing.expectEqual(@as(u32, 0), conid);
    try std.testing.expectEqual(@as(u32, 1), crm.getContactCount(cid));
}

// ============ Assets Tests ============

test "asset manager total value" {
    const allocator = std.testing.allocator;
    var mgr = assets_module.AssetManager.init(allocator);
    defer mgr.deinit();

    _ = try mgr.addAsset("Laptop", "Dev", assets_module.AssetType.equipment, 1000.0, 0);
    _ = try mgr.addAsset("Server", "Hosting", assets_module.AssetType.equipment, 2000.0, 0);
    try std.testing.expectEqual(3000.0, mgr.getTotalValue());
}

test "deactivate worker" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    const worker_id = try manager.registerWorker("Alice", "alice@example.com", 75.50);
    try manager.deactivateWorker(worker_id);

    try std.testing.expectEqual(@as(u32, 0), manager.getActiveCount());
    try std.testing.expect(!manager.workers.items[0].active);
}

test "worker account management" {
    const allocator = std.testing.allocator;
    var manager = worker_module.WorkerManager.init(allocator);
    defer manager.deinit();

    const wid = try manager.registerWorker("Alice", "alice@example.com", 75.50, worker_module.WorkerType.freelancer);
    const acct1 = try manager.addWorkerAccount(wid, accounts_module.AccountType.social_media, "alice123", "twitter", "https://twitter.com/alice");
    const acct2 = try manager.addWorkerAccount(wid, accounts_module.AccountType.email, "alice@example.com", "gmail", "primary");

    try std.testing.expectEqual(@as(u32, 0), acct1);
    try std.testing.expectEqual(@as(u32, 1), acct2);
    try std.testing.expectEqual(@as(u32, 2), manager.getWorkerActiveAccountCount(wid));

    try manager.deactivateWorkerAccount(wid, acct1);
    try std.testing.expectEqual(@as(u32, 1), manager.getWorkerActiveAccountCount(wid));
}

// ============ Portfolio Module Tests ============

test "PortfolioManager initialization" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try std.testing.expect(manager.portfolio == null);
    try std.testing.expectEqual(@as(usize, 0), manager.index.items.len);
    try std.testing.expectEqual(@as(u32, 0), manager.next_id);
}

test "create portfolio" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");

    try std.testing.expect(manager.portfolio != null);
    try std.testing.expectEqualStrings("Main Portfolio", manager.portfolio.?.name);
    try std.testing.expectEqualStrings("Strategic portfolio", manager.portfolio.?.description);
    try std.testing.expectEqualStrings("enterprise", manager.portfolio.?.metadata.category);
}

test "add sub-portfolio" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addSubPortfolio("Sub Portfolio 1", "Sub portfolio description", "division");

    try std.testing.expectEqual(@as(usize, 1), manager.portfolio.?.sub_portfolios.items.len);
    const sub = manager.portfolio.?.sub_portfolios.items[0];
    try std.testing.expectEqualStrings("Sub Portfolio 1", sub.name);
}

test "add multiple sub-portfolios" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addSubPortfolio("Sub 1", "Sub portfolio 1", "division");
    try manager.addSubPortfolio("Sub 2", "Sub portfolio 2", "division");
    try manager.addSubPortfolio("Sub 3", "Sub portfolio 3", "division");

    try std.testing.expectEqual(@as(usize, 3), manager.portfolio.?.sub_portfolios.items.len);
}

test "add program to portfolio" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProgram("Program 1", "Program description", "development");

    try std.testing.expectEqual(@as(usize, 1), manager.portfolio.?.programs.items.len);
    const program = manager.portfolio.?.programs.items[0];
    try std.testing.expectEqualStrings("Program 1", program.name);
    try std.testing.expectEqual(portfolio_module.EntityType.program, program.metadata.entity_type);
}

test "add project to portfolio" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProject("Project 1", "Project description", "development");

    try std.testing.expectEqual(@as(usize, 1), manager.portfolio.?.projects.items.len);
    const project = manager.portfolio.?.projects.items[0];
    try std.testing.expectEqualStrings("Project 1", project.name);
    try std.testing.expectEqual(portfolio_module.EntityType.project, project.metadata.entity_type);
}

test "add task to project" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProject("Project 1", "Project description", "development");
    try manager.addTask(1, "Task 1", "Task description");

    try std.testing.expectEqual(@as(usize, 1), manager.portfolio.?.projects.items[0].tasks.items.len);
    const task = manager.portfolio.?.projects.items[0].tasks.items[0];
    try std.testing.expectEqualStrings("Task 1", task.title);
    try std.testing.expectEqual(portfolio_module.EntityType.task, task.metadata.entity_type);
}

test "search portfolio by name pattern" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProgram("Development Program", "Dev program", "development");
    try manager.addProgram("Testing Program", "Test program", "testing");
    try manager.addProject("Core Project", "Core project", "development");

    var filter = portfolio_module.SearchFilter{
        .name_pattern = "Program",
    };
    var results = try manager.search(filter, allocator);
    defer {
        for (results.items) |*item| {
            allocator.free(item.name);
        }
        results.deinit(allocator);
    }

    try std.testing.expectEqual(@as(usize, 2), results.items.len);
}

test "search portfolio by entity type" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProgram("Program 1", "Program 1", "development");
    try manager.addProgram("Program 2", "Program 2", "testing");
    try manager.addProject("Project 1", "Project 1", "development");

    var filter = portfolio_module.SearchFilter{
        .entity_type = portfolio_module.EntityType.program,
    };
    var results = try manager.search(filter, allocator);
    defer {
        for (results.items) |*item| {
            allocator.free(item.name);
        }
        results.deinit(allocator);
    }

    try std.testing.expectEqual(@as(usize, 2), results.items.len);
}

test "get entities by type" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addProgram("Program 1", "Program 1", "development");
    try manager.addProject("Project 1", "Project 1", "development");
    try manager.addProject("Project 2", "Project 2", "testing");

    var results = try manager.getEntitiesByType(portfolio_module.EntityType.project, allocator);
    defer {
        for (results.items) |*item| {
            allocator.free(item.name);
        }
        results.deinit(allocator);
    }

    try std.testing.expectEqual(@as(usize, 2), results.items.len);
}

test "add tag to entity" {
    const allocator = std.testing.allocator;
    var manager = portfolio_module.PortfolioManager.init(allocator);
    defer manager.deinit();

    try manager.createPortfolio("Main Portfolio", "Strategic portfolio", "enterprise");
    try manager.addTagToEntity(0, "urgent", "priority");

    const metadata = manager.portfolio.?.metadata;
    try std.testing.expectEqual(@as(usize, 1), metadata.tags.items.len);
    try std.testing.expectEqualStrings("urgent", metadata.tags.items[0].name);
    try std.testing.expectEqualStrings("priority", metadata.tags.items[0].category);
}

// ============ Kernel Module Tests ============

test "Kernel initialization" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    try std.testing.expectEqual(@as(usize, 0), kernel.jobs.items.len);
    try std.testing.expectEqual(@as(usize, 0), kernel.contracts.items.len);
    try std.testing.expectEqual(@as(u32, 0), kernel.getActiveWorkersCount());
}

test "register worker through kernel" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker_id = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    try std.testing.expectEqual(@as(u32, 0), worker_id);
    try std.testing.expectEqual(@as(u32, 1), kernel.getActiveWorkersCount());
}

test "post job" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    try std.testing.expectEqual(@as(u32, 0), job_id);
    try std.testing.expectEqual(@as(usize, 1), kernel.jobs.items.len);

    const job = kernel.jobs.items[0];
    try std.testing.expectEqualStrings("Build API", job.title);
    try std.testing.expectEqual(@as(f32, 5000.00), job.budget);
}

test "add skill requirement to job" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    try kernel.addJobSkillRequirement(job_id, "Zig Programming");
    try kernel.addJobSkillRequirement(job_id, "System Design");

    const job = kernel.jobs.items[0];
    try std.testing.expectEqual(@as(usize, 2), job.required_skills.items.len);
}

test "create contract" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker_id = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    const contract_id = try kernel.createContract(worker_id, job_id, 1735000000, 75.50);

    try std.testing.expectEqual(@as(u32, 0), contract_id);
    try std.testing.expectEqual(@as(usize, 1), kernel.contracts.items.len);

    const contract = kernel.contracts.items[0];
    try std.testing.expectEqual(worker_id, contract.worker_id);
    try std.testing.expectEqual(job_id, contract.job_id);
    try std.testing.expectEqual(@as(f32, 75.50), contract.hourly_rate);
}

test "log hours on contract" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker_id = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    const contract_id = try kernel.createContract(worker_id, job_id, 1735000000, 75.50);

    try kernel.logHours(contract_id, 8.0);
    try std.testing.expectEqual(@as(f32, 8.0), kernel.contracts.items[0].hours_worked);

    try kernel.logHours(contract_id, 4.5);
    try std.testing.expectEqual(@as(f32, 12.5), kernel.contracts.items[0].hours_worked);
}

test "calculate worker earnings" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker_id = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    const contract_id = try kernel.createContract(worker_id, job_id, 1735000000, 75.50);

    try kernel.logHours(contract_id, 32.5);

    const earnings = kernel.calculateWorkerEarnings(worker_id);
    const expected_earnings = 32.5 * 75.50;
    try std.testing.expectApproxEqAbs(expected_earnings, earnings, 0.01);
}

test "complete contract" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker_id = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const job_id = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    const contract_id = try kernel.createContract(worker_id, job_id, 1735000000, 75.50);

    try kernel.logHours(contract_id, 32.5);
    try kernel.completeContract(contract_id, 1735100000);

    const contract = kernel.contracts.items[0];
    try std.testing.expect(contract.end_date != null);
    try std.testing.expectEqual(kernel_module.Contract.status.completed, contract.status);

    const job = kernel.jobs.items[0];
    try std.testing.expect(job.completed);
}

test "multiple workers and contracts" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    const worker1 = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const worker2 = try kernel.registerWorker("Bob", "bob@example.com", 65.00);

    const job1 = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    const job2 = try kernel.postJob("Write Tests", "Unit tests", 2000.00, 1735000000);

    const contract1 = try kernel.createContract(worker1, job1, 1735000000, 75.50);
    const contract2 = try kernel.createContract(worker2, job2, 1735000000, 65.00);

    try kernel.logHours(contract1, 32.5);
    try kernel.logHours(contract2, 24.0);

    const earnings1 = kernel.calculateWorkerEarnings(worker1);
    const earnings2 = kernel.calculateWorkerEarnings(worker2);

    try std.testing.expectApproxEqAbs(32.5 * 75.50, earnings1, 0.01);
    try std.testing.expectApproxEqAbs(24.0 * 65.00, earnings2, 0.01);
}

test "get active jobs count" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    try std.testing.expectEqual(@as(u32, 0), kernel.getActiveJobsCount());

    const job1 = try kernel.postJob("Build API", "REST API", 5000.00, 1735000000);
    try std.testing.expectEqual(@as(u32, 1), kernel.getActiveJobsCount());

    const job2 = try kernel.postJob("Write Tests", "Unit tests", 2000.00, 1735000000);
    try std.testing.expectEqual(@as(u32, 2), kernel.getActiveJobsCount());

    const worker = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    const contract = try kernel.createContract(worker, job1, 1735000000, 75.50);
    try kernel.completeContract(contract, 1735100000);

    try std.testing.expectEqual(@as(u32, 1), kernel.getActiveJobsCount());
}

test "get workers through kernel" {
    const allocator = std.testing.allocator;
    var kernel = kernel_module.Kernel.init(allocator);
    defer kernel.deinit();

    _ = try kernel.registerWorker("Alice", "alice@example.com", 75.50);
    _ = try kernel.registerWorker("Bob", "bob@example.com", 65.00);

    const workers = kernel.getWorkers();
    try std.testing.expectEqual(@as(usize, 2), workers.len);
    try std.testing.expectEqualStrings("Alice", workers[0].name);
    try std.testing.expectEqualStrings("Bob", workers[1].name);
}
