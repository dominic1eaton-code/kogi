const std = @import("std");

pub const ProjectStatus = enum {
    proposed,
    planned,
    active,
    blocked,
    completed,
    canceled,
    archived,
};

pub const ProjectPriority = enum {
    low,
    medium,
    high,
    critical,
};

pub const MilestoneStatus = enum {
    pending,
    in_progress,
    completed,
    delayed,
    canceled,
};

pub const TaskStatus = enum {
    todo,
    in_progress,
    blocked,
    done,
    canceled,
};

pub const IssueSeverity = enum {
    low,
    medium,
    high,
    critical,
};

pub const ProgressUpdateType = enum {
    general,
    scope,
    timeline,
    budget,
    risk,
    decision,
};

pub const StoryType = enum {
    feature,
    bug,
    capability,
    enabler,
    blocker,
    @"test",
    requirement,
    defect,
    issue,
    review,
    audit,
    report,
    performance,
    strategy,
    tactic,
    operation,
    business_case,
    reqruiement,
    research,
    prototype,
    spike,
    idea,
    concept,
    design,
    documentation,
    milestone,
    objective,
    outcome,
    risk,
    mission,
    vision,
    goal,
};

pub const StoryStatus = enum {
    backlog,
    ready,
    in_progress,
    blocked,
    in_review,
    done,
    canceled,
};

pub const ProjectMilestone = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    due_at: ?i64,
    status: MilestoneStatus,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectTask = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    assignee_id: ?u32,
    due_at: ?i64,
    status: TaskStatus,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectIssue = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    severity: IssueSeverity,
    is_resolved: bool,
    created_at: i64,
    resolved_at: ?i64 = null,
};

pub const ProjectNote = struct {
    id: u32,
    title: []const u8,
    body: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectProgressUpdate = struct {
    id: u32,
    update_type: ProgressUpdateType,
    message: []const u8,
    created_at: i64,
    author_id: ?u32 = null,
};

pub const Story = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    story_type: StoryType,
    status: StoryStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    estimate_points: f32,
    created_at: i64,
    updated_at: i64,
};

pub const TrackedProject = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    owner_id: ?u32,
    status: ProjectStatus,
    priority: ProjectPriority,
    created_at: i64,
    updated_at: i64,
    start_at: ?i64,
    target_end_at: ?i64,
    actual_end_at: ?i64 = null,
    tags: std.array_list.Managed([]const u8),
    milestones: std.array_list.Managed(ProjectMilestone),
    tasks: std.array_list.Managed(ProjectTask),
    issues: std.array_list.Managed(ProjectIssue),
    stories: std.array_list.Managed(Story),
    notes: std.array_list.Managed(ProjectNote),
    updates: std.array_list.Managed(ProjectProgressUpdate),
};

pub const ProjectFilter = struct {
    status: ?ProjectStatus = null,
    priority: ?ProjectPriority = null,
    owner_id: ?u32 = null,
    tag_name: ?[]const u8 = null,
    name_pattern: ?[]const u8 = null,
    active_on: ?i64 = null,
    include_archived: bool = false,
};

pub const ProjectSummary = struct {
    id: u32,
    name: []const u8,
    status: ProjectStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    progress_percent: f32,
    open_issues: u32,
    target_end_at: ?i64,
    updated_at: i64,
};

pub const StoryFilter = struct {
    story_type: ?StoryType = null,
    status: ?StoryStatus = null,
    priority: ?ProjectPriority = null,
    owner_id: ?u32 = null,
    query: ?[]const u8 = null,
    include_done: bool = true,
};

pub const StorySummary = struct {
    id: u32,
    title: []const u8,
    story_type: StoryType,
    status: StoryStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    updated_at: i64,
};

pub const ProjectTrackingManager = struct {
    allocator: std.mem.Allocator,
    projects: std.array_list.Managed(TrackedProject),
    next_project_id: u32 = 1,

    pub fn init(allocator: std.mem.Allocator) ProjectTrackingManager {
        return .{
            .allocator = allocator,
            .projects = std.array_list.Managed(TrackedProject).init(allocator),
        };
    }

    pub fn deinit(self: *ProjectTrackingManager) void {
        for (self.projects.items) |*project| {
            self.deinitProject(project);
        }
        self.projects.deinit();
    }

    fn deinitProject(self: *ProjectTrackingManager, project: *TrackedProject) void {
        self.allocator.free(project.name);
        self.allocator.free(project.description);

        for (project.tags.items) |tag| {
            self.allocator.free(tag);
        }
        project.tags.deinit();

        for (project.milestones.items) |*milestone| {
            self.allocator.free(milestone.name);
            self.allocator.free(milestone.description);
        }
        project.milestones.deinit();

        for (project.tasks.items) |*task| {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
        }
        project.tasks.deinit();

        for (project.issues.items) |*issue| {
            self.allocator.free(issue.title);
            self.allocator.free(issue.description);
        }
        project.issues.deinit();

        for (project.stories.items) |*story| {
            self.allocator.free(story.title);
            self.allocator.free(story.description);
        }
        project.stories.deinit();

        for (project.notes.items) |*note| {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
        }
        project.notes.deinit();

        for (project.updates.items) |*update| {
            self.allocator.free(update.message);
        }
        project.updates.deinit();
    }

    fn getProjectMutable(self: *ProjectTrackingManager, project_id: u32) ?*TrackedProject {
        for (self.projects.items) |*project| {
            if (project.id == project_id) return project;
        }
        return null;
    }

    fn projectHasTag(project: *const TrackedProject, tag_name: []const u8) bool {
        for (project.tags.items) |tag| {
            if (std.mem.eql(u8, tag, tag_name)) return true;
        }
        return false;
    }

    fn openIssueCount(project: *const TrackedProject) u32 {
        var count: u32 = 0;
        for (project.issues.items) |issue| {
            if (!issue.is_resolved) count += 1;
        }
        return count;
    }

    fn matchesFilter(project: *const TrackedProject, filter: ProjectFilter) bool {
        if (!filter.include_archived and project.status == .archived) return false;

        if (filter.status) |status| {
            if (project.status != status) return false;
        }
        if (filter.priority) |priority| {
            if (project.priority != priority) return false;
        }
        if (filter.owner_id) |owner_id| {
            if (project.owner_id != owner_id) return false;
        }
        if (filter.tag_name) |tag_name| {
            if (!projectHasTag(project, tag_name)) return false;
        }
        if (filter.name_pattern) |name_pattern| {
            if (std.mem.indexOf(u8, project.name, name_pattern) == null) return false;
        }
        if (filter.active_on) |ts| {
            if (project.start_at) |start| {
                if (ts < start) return false;
            }
            const end_bound = project.actual_end_at orelse project.target_end_at;
            if (end_bound) |end| {
                if (ts > end) return false;
            }
        }
        return true;
    }

    fn matchesStoryFilter(story: *const Story, filter: StoryFilter) bool {
        if (!filter.include_done and story.status == .done) return false;

        if (filter.story_type) |story_type| {
            if (story.story_type != story_type) return false;
        }
        if (filter.status) |status| {
            if (story.status != status) return false;
        }
        if (filter.priority) |priority| {
            if (story.priority != priority) return false;
        }
        if (filter.owner_id) |owner_id| {
            if (story.owner_id != owner_id) return false;
        }
        if (filter.query) |query| {
            const in_title = std.mem.indexOf(u8, story.title, query) != null;
            const in_description = std.mem.indexOf(u8, story.description, query) != null;
            if (!in_title and !in_description) return false;
        }

        return true;
    }

    pub fn createProject(
        self: *ProjectTrackingManager,
        name: []const u8,
        description: []const u8,
        owner_id: ?u32,
        priority: ProjectPriority,
        start_at: ?i64,
        target_end_at: ?i64,
        created_at: i64,
    ) !u32 {
        const id = self.next_project_id;
        self.next_project_id += 1;

        var project = TrackedProject{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .owner_id = owner_id,
            .status = .proposed,
            .priority = priority,
            .created_at = created_at,
            .updated_at = created_at,
            .start_at = start_at,
            .target_end_at = target_end_at,
            .tags = std.array_list.Managed([]const u8).init(self.allocator),
            .milestones = std.array_list.Managed(ProjectMilestone).init(self.allocator),
            .tasks = std.array_list.Managed(ProjectTask).init(self.allocator),
            .issues = std.array_list.Managed(ProjectIssue).init(self.allocator),
            .stories = std.array_list.Managed(Story).init(self.allocator),
            .notes = std.array_list.Managed(ProjectNote).init(self.allocator),
            .updates = std.array_list.Managed(ProjectProgressUpdate).init(self.allocator),
        };

        errdefer self.deinitProject(&project);
        try self.projects.append(project);
        return id;
    }

    pub fn updateProjectStatus(self: *ProjectTrackingManager, project_id: u32, status: ProjectStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        project.status = status;
        project.updated_at = updated_at;
        if (status == .completed or status == .canceled or status == .archived) {
            project.actual_end_at = updated_at;
        }
    }

    pub fn setProjectSchedule(self: *ProjectTrackingManager, project_id: u32, start_at: ?i64, target_end_at: ?i64, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        project.start_at = start_at;
        project.target_end_at = target_end_at;
        project.updated_at = updated_at;
    }

    pub fn addProjectTag(self: *ProjectTrackingManager, project_id: u32, tag_name: []const u8, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (projectHasTag(project, tag_name)) return;

        try project.tags.append(try self.allocator.dupe(u8, tag_name));
        project.updated_at = updated_at;
    }

    pub fn addMilestone(
        self: *ProjectTrackingManager,
        project_id: u32,
        name: []const u8,
        description: []const u8,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const milestone_id = @as(u32, @intCast(project.milestones.items.len));

        const milestone = ProjectMilestone{
            .id = milestone_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .due_at = due_at,
            .status = .pending,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(milestone.name);
            self.allocator.free(milestone.description);
        }

        try project.milestones.append(milestone);
        project.updated_at = created_at;
        return milestone_id;
    }

    pub fn setMilestoneStatus(self: *ProjectTrackingManager, project_id: u32, milestone_id: u32, status: MilestoneStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (milestone_id >= @as(u32, @intCast(project.milestones.items.len))) return error.MilestoneNotFound;

        const milestone = &project.milestones.items[milestone_id];
        milestone.status = status;
        milestone.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addTask(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        assignee_id: ?u32,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const task_id = @as(u32, @intCast(project.tasks.items.len));

        const task = ProjectTask{
            .id = task_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .assignee_id = assignee_id,
            .due_at = due_at,
            .status = .todo,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
        }

        try project.tasks.append(task);
        project.updated_at = created_at;
        return task_id;
    }

    pub fn setTaskStatus(self: *ProjectTrackingManager, project_id: u32, task_id: u32, status: TaskStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (task_id >= @as(u32, @intCast(project.tasks.items.len))) return error.TaskNotFound;

        const task = &project.tasks.items[task_id];
        task.status = status;
        task.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addStory(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        story_type: StoryType,
        priority: ProjectPriority,
        owner_id: ?u32,
        estimate_points: f32,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const story_id = @as(u32, @intCast(project.stories.items.len));

        const story = Story{
            .id = story_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .story_type = story_type,
            .status = .backlog,
            .priority = priority,
            .owner_id = owner_id,
            .estimate_points = estimate_points,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(story.title);
            self.allocator.free(story.description);
        }

        try project.stories.append(story);
        project.updated_at = created_at;
        return story_id;
    }

    pub fn setStoryStatus(self: *ProjectTrackingManager, project_id: u32, story_id: u32, status: StoryStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (story_id >= @as(u32, @intCast(project.stories.items.len))) return error.StoryNotFound;

        const story = &project.stories.items[story_id];
        story.status = status;
        story.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn updateStoryType(self: *ProjectTrackingManager, project_id: u32, story_id: u32, story_type: StoryType, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (story_id >= @as(u32, @intCast(project.stories.items.len))) return error.StoryNotFound;

        const story = &project.stories.items[story_id];
        story.story_type = story_type;
        story.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn getStories(self: *ProjectTrackingManager, project_id: u32) ![]Story {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        return project.stories.items;
    }

    pub fn filterStories(
        self: *ProjectTrackingManager,
        project_id: u32,
        filter: StoryFilter,
        allocator: std.mem.Allocator,
    ) !std.array_list.Managed(StorySummary) {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        var results = std.array_list.Managed(StorySummary).init(allocator);

        for (project.stories.items) |*story| {
            if (!matchesStoryFilter(story, filter)) continue;

            const summary = StorySummary{
                .id = story.id,
                .title = try allocator.dupe(u8, story.title),
                .story_type = story.story_type,
                .status = story.status,
                .priority = story.priority,
                .owner_id = story.owner_id,
                .updated_at = story.updated_at,
            };
            errdefer allocator.free(summary.title);
            try results.append(summary);
        }

        return results;
    }

    pub fn deinitStorySummaries(results: *std.array_list.Managed(StorySummary), allocator: std.mem.Allocator) void {
        for (results.items) |summary| {
            allocator.free(summary.title);
        }
        results.deinit();
    }

    pub fn addIssue(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        severity: IssueSeverity,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const issue_id = @as(u32, @intCast(project.issues.items.len));

        const issue = ProjectIssue{
            .id = issue_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .severity = severity,
            .is_resolved = false,
            .created_at = created_at,
        };
        errdefer {
            self.allocator.free(issue.title);
            self.allocator.free(issue.description);
        }

        try project.issues.append(issue);
        project.updated_at = created_at;
        return issue_id;
    }

    pub fn setIssueResolved(self: *ProjectTrackingManager, project_id: u32, issue_id: u32, resolved: bool, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (issue_id >= @as(u32, @intCast(project.issues.items.len))) return error.IssueNotFound;

        const issue = &project.issues.items[issue_id];
        issue.is_resolved = resolved;
        issue.resolved_at = if (resolved) updated_at else null;
        project.updated_at = updated_at;
    }

    pub fn addNote(self: *ProjectTrackingManager, project_id: u32, title: []const u8, body: []const u8, created_at: i64) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const note_id = @as(u32, @intCast(project.notes.items.len));

        const note = ProjectNote{
            .id = note_id,
            .title = try self.allocator.dupe(u8, title),
            .body = try self.allocator.dupe(u8, body),
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
        }

        try project.notes.append(note);
        project.updated_at = created_at;
        return note_id;
    }

    pub fn updateNote(self: *ProjectTrackingManager, project_id: u32, note_id: u32, title: []const u8, body: []const u8, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (note_id >= @as(u32, @intCast(project.notes.items.len))) return error.NoteNotFound;

        const note = &project.notes.items[note_id];
        const new_title = try self.allocator.dupe(u8, title);
        errdefer self.allocator.free(new_title);
        const new_body = try self.allocator.dupe(u8, body);
        errdefer self.allocator.free(new_body);

        self.allocator.free(note.title);
        self.allocator.free(note.body);
        note.title = new_title;
        note.body = new_body;
        note.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addProgressUpdate(
        self: *ProjectTrackingManager,
        project_id: u32,
        update_type: ProgressUpdateType,
        message: []const u8,
        created_at: i64,
        author_id: ?u32,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const update_id = @as(u32, @intCast(project.updates.items.len));

        const update = ProjectProgressUpdate{
            .id = update_id,
            .update_type = update_type,
            .message = try self.allocator.dupe(u8, message),
            .created_at = created_at,
            .author_id = author_id,
        };
        errdefer self.allocator.free(update.message);

        try project.updates.append(update);
        project.updated_at = created_at;
        return update_id;
    }

    pub fn calculateProgress(self: *ProjectTrackingManager, project_id: u32) !f32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;

        const milestone_count = project.milestones.items.len;
        var completed_milestones: usize = 0;
        for (project.milestones.items) |milestone| {
            if (milestone.status == .completed) completed_milestones += 1;
        }

        const task_count = project.tasks.items.len;
        var completed_tasks: usize = 0;
        for (project.tasks.items) |task| {
            if (task.status == .done) completed_tasks += 1;
        }

        const story_count = project.stories.items.len;
        var completed_stories: usize = 0;
        for (project.stories.items) |story| {
            if (story.status == .done) completed_stories += 1;
        }

        if (milestone_count == 0 and task_count == 0 and story_count == 0) return 0.0;

        const milestone_progress: f32 = if (milestone_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_milestones)) / @as(f32, @floatFromInt(milestone_count));

        const task_progress: f32 = if (task_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_tasks)) / @as(f32, @floatFromInt(task_count));

        const story_progress: f32 = if (story_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_stories)) / @as(f32, @floatFromInt(story_count));

        if (milestone_count == 0 and task_count == 0) return story_progress * 100.0;
        if (milestone_count == 0 and story_count == 0) return task_progress * 100.0;
        if (task_count == 0 and story_count == 0) return milestone_progress * 100.0;

        return ((milestone_progress * 0.4) + (task_progress * 0.3) + (story_progress * 0.3)) * 100.0;
    }

    pub fn getProjectById(self: *ProjectTrackingManager, project_id: u32) ?*TrackedProject {
        return self.getProjectMutable(project_id);
    }

    pub fn listProjects(self: *ProjectTrackingManager) []TrackedProject {
        return self.projects.items;
    }

    pub fn filterProjects(self: *ProjectTrackingManager, filter: ProjectFilter, allocator: std.mem.Allocator) !std.array_list.Managed(ProjectSummary) {
        var results = std.array_list.Managed(ProjectSummary).init(allocator);

        for (self.projects.items) |*project| {
            if (!matchesFilter(project, filter)) continue;

            const summary = ProjectSummary{
                .id = project.id,
                .name = try allocator.dupe(u8, project.name),
                .status = project.status,
                .priority = project.priority,
                .owner_id = project.owner_id,
                .progress_percent = try self.calculateProgress(project.id),
                .open_issues = openIssueCount(project),
                .target_end_at = project.target_end_at,
                .updated_at = project.updated_at,
            };
            errdefer allocator.free(summary.name);

            try results.append(summary);
        }

        return results;
    }

    pub fn deinitProjectSummaries(results: *std.array_list.Managed(ProjectSummary), allocator: std.mem.Allocator) void {
        for (results.items) |summary| {
            allocator.free(summary.name);
        }
        results.deinit();
    }
};

pub fn projectTrackingDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = ProjectTrackingManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Project tracking manager initialized\n", .{});
}
