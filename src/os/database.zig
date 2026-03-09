const std = @import("std");

pub const ColumnType = enum {
    text,
    integer,
    float,
    boolean,
    json,
};

pub const TransactionOp = enum {
    insert,
    update,
    delete,
};

pub const RecordField = struct {
    key: []const u8,
    value: []const u8,
};

pub const TableColumn = struct {
    name: []const u8,
    column_type: ColumnType,
    nullable: bool,
    default_value: []const u8,
};

pub const Database = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const Table = struct {
    id: u32,
    database_id: u32,
    name: []const u8,
    description: []const u8,
    columns: std.array_list.Managed(TableColumn),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const Record = struct {
    id: u32,
    database_id: u32,
    table_id: u32,
    fields: std.array_list.Managed(RecordField),
    version: u32,
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const Transaction = struct {
    id: u32,
    database_id: u32,
    table_id: u32,
    record_id: u32,
    op: TransactionOp,
    summary: []const u8,
    committed_at: i64,
};

pub const DatabaseError = error{
    DatabaseNotFound,
    TableNotFound,
    RecordNotFound,
    ColumnNotFound,
};

pub const DatabaseManager = struct {
    allocator: std.mem.Allocator,
    databases: std.array_list.Managed(Database),
    tables: std.array_list.Managed(Table),
    records: std.array_list.Managed(Record),
    transactions: std.array_list.Managed(Transaction),
    next_database_id: u32,
    next_table_id: u32,
    next_record_id: u32,
    next_transaction_id: u32,

    pub fn init(allocator: std.mem.Allocator) DatabaseManager {
        return DatabaseManager{
            .allocator = allocator,
            .databases = std.array_list.Managed(Database).init(allocator),
            .tables = std.array_list.Managed(Table).init(allocator),
            .records = std.array_list.Managed(Record).init(allocator),
            .transactions = std.array_list.Managed(Transaction).init(allocator),
            .next_database_id = 0,
            .next_table_id = 0,
            .next_record_id = 0,
            .next_transaction_id = 0,
        };
    }

    pub fn deinit(self: *DatabaseManager) void {
        for (self.databases.items) |db| {
            self.allocator.free(db.name);
            self.allocator.free(db.description);
        }
        self.databases.deinit();

        for (self.tables.items) |*table| {
            self.allocator.free(table.name);
            self.allocator.free(table.description);
            for (table.columns.items) |col| {
                self.allocator.free(col.name);
                self.allocator.free(col.default_value);
            }
            table.columns.deinit();
        }
        self.tables.deinit();

        for (self.records.items) |*record| {
            for (record.fields.items) |field| {
                self.allocator.free(field.key);
                self.allocator.free(field.value);
            }
            record.fields.deinit();
        }
        self.records.deinit();

        for (self.transactions.items) |txn| {
            self.allocator.free(txn.summary);
        }
        self.transactions.deinit();
    }

    pub fn createDatabase(self: *DatabaseManager, name: []const u8, description: []const u8) !u32 {
        const id = self.next_database_id;
        self.next_database_id += 1;
        const now = std.time.timestamp();
        try self.databases.append(.{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .active = true,
            .created_at = now,
            .updated_at = now,
        });
        return id;
    }

    pub fn updateDatabase(self: *DatabaseManager, database_id: u32, name: []const u8, description: []const u8) !void {
        const idx = self.findDatabaseIndexById(database_id) orelse return DatabaseError.DatabaseNotFound;
        self.allocator.free(self.databases.items[idx].name);
        self.allocator.free(self.databases.items[idx].description);
        self.databases.items[idx].name = try self.allocator.dupe(u8, name);
        self.databases.items[idx].description = try self.allocator.dupe(u8, description);
        self.databases.items[idx].updated_at = std.time.timestamp();
    }

    pub fn createTable(self: *DatabaseManager, database_id: u32, name: []const u8, description: []const u8) !u32 {
        _ = self.findDatabaseIndexById(database_id) orelse return DatabaseError.DatabaseNotFound;
        const id = self.next_table_id;
        self.next_table_id += 1;
        const now = std.time.timestamp();
        try self.tables.append(.{
            .id = id,
            .database_id = database_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .columns = std.array_list.Managed(TableColumn).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        });
        return id;
    }

    pub fn addTableColumn(
        self: *DatabaseManager,
        table_id: u32,
        name: []const u8,
        column_type: ColumnType,
        nullable: bool,
        default_value: []const u8,
    ) !void {
        const idx = self.findTableIndexById(table_id) orelse return DatabaseError.TableNotFound;
        try self.tables.items[idx].columns.append(.{
            .name = try self.allocator.dupe(u8, name),
            .column_type = column_type,
            .nullable = nullable,
            .default_value = try self.allocator.dupe(u8, default_value),
        });
        self.tables.items[idx].updated_at = std.time.timestamp();
    }

    pub fn createRecord(self: *DatabaseManager, database_id: u32, table_id: u32) !u32 {
        _ = self.findDatabaseIndexById(database_id) orelse return DatabaseError.DatabaseNotFound;
        const table_idx = self.findTableIndexById(table_id) orelse return DatabaseError.TableNotFound;
        if (self.tables.items[table_idx].database_id != database_id) return DatabaseError.TableNotFound;

        const id = self.next_record_id;
        self.next_record_id += 1;
        const now = std.time.timestamp();
        try self.records.append(.{
            .id = id,
            .database_id = database_id,
            .table_id = table_id,
            .fields = std.array_list.Managed(RecordField).init(self.allocator),
            .version = 1,
            .active = true,
            .created_at = now,
            .updated_at = now,
        });
        _ = try self.commitTransaction(database_id, table_id, id, .insert, "record inserted");
        return id;
    }

    pub fn setRecordField(self: *DatabaseManager, record_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findRecordIndexById(record_id) orelse return DatabaseError.RecordNotFound;
        const record = &self.records.items[idx];
        for (record.fields.items) |*field| {
            if (std.mem.eql(u8, field.key, key)) {
                self.allocator.free(field.value);
                field.value = try self.allocator.dupe(u8, value);
                record.version += 1;
                record.updated_at = std.time.timestamp();
                _ = try self.commitTransaction(record.database_id, record.table_id, record.id, .update, "record field updated");
                return;
            }
        }
        try record.fields.append(.{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
        });
        record.version += 1;
        record.updated_at = std.time.timestamp();
        _ = try self.commitTransaction(record.database_id, record.table_id, record.id, .update, "record field inserted");
    }

    pub fn deleteRecord(self: *DatabaseManager, record_id: u32) !void {
        const idx = self.findRecordIndexById(record_id) orelse return DatabaseError.RecordNotFound;
        self.records.items[idx].active = false;
        self.records.items[idx].version += 1;
        self.records.items[idx].updated_at = std.time.timestamp();
        _ = try self.commitTransaction(
            self.records.items[idx].database_id,
            self.records.items[idx].table_id,
            self.records.items[idx].id,
            .delete,
            "record soft deleted",
        );
    }

    pub fn getDatabaseById(self: *DatabaseManager, database_id: u32) !Database {
        const idx = self.findDatabaseIndexById(database_id) orelse return DatabaseError.DatabaseNotFound;
        return self.databases.items[idx];
    }

    pub fn getTableById(self: *DatabaseManager, table_id: u32) !Table {
        const idx = self.findTableIndexById(table_id) orelse return DatabaseError.TableNotFound;
        return self.tables.items[idx];
    }

    pub fn getRecordById(self: *DatabaseManager, record_id: u32) !Record {
        const idx = self.findRecordIndexById(record_id) orelse return DatabaseError.RecordNotFound;
        return self.records.items[idx];
    }

    pub fn getDatabases(self: *DatabaseManager) []Database {
        return self.databases.items;
    }

    pub fn getTables(self: *DatabaseManager) []Table {
        return self.tables.items;
    }

    pub fn getRecords(self: *DatabaseManager) []Record {
        return self.records.items;
    }

    pub fn queryRecordsByField(
        self: *DatabaseManager,
        table_id: u32,
        key: []const u8,
        value: []const u8,
        allocator: std.mem.Allocator,
    ) !std.array_list.Managed(Record) {
        _ = self.findTableIndexById(table_id) orelse return DatabaseError.TableNotFound;
        var results = std.array_list.Managed(Record).init(allocator);
        for (self.records.items) |record| {
            if (record.table_id != table_id or !record.active) continue;
            for (record.fields.items) |field| {
                if (std.mem.eql(u8, field.key, key) and std.mem.eql(u8, field.value, value)) {
                    try results.append(record);
                    break;
                }
            }
        }
        return results;
    }

    pub fn getTransactions(self: *DatabaseManager) []Transaction {
        return self.transactions.items;
    }

    pub fn commitTransaction(
        self: *DatabaseManager,
        database_id: u32,
        table_id: u32,
        record_id: u32,
        op: TransactionOp,
        summary: []const u8,
    ) !u32 {
        const id = self.next_transaction_id;
        self.next_transaction_id += 1;
        try self.transactions.append(.{
            .id = id,
            .database_id = database_id,
            .table_id = table_id,
            .record_id = record_id,
            .op = op,
            .summary = try self.allocator.dupe(u8, summary),
            .committed_at = std.time.timestamp(),
        });
        return id;
    }

    fn findDatabaseIndexById(self: *DatabaseManager, database_id: u32) ?usize {
        for (self.databases.items, 0..) |db, idx| {
            if (db.id == database_id) return idx;
        }
        return null;
    }

    fn findTableIndexById(self: *DatabaseManager, table_id: u32) ?usize {
        for (self.tables.items, 0..) |table, idx| {
            if (table.id == table_id) return idx;
        }
        return null;
    }

    fn findRecordIndexById(self: *DatabaseManager, record_id: u32) ?usize {
        for (self.records.items, 0..) |record, idx| {
            if (record.id == record_id) return idx;
        }
        return null;
    }
};

pub fn parseColumnType(name: []const u8) ?ColumnType {
    if (std.mem.eql(u8, name, "text")) return .text;
    if (std.mem.eql(u8, name, "integer")) return .integer;
    if (std.mem.eql(u8, name, "float")) return .float;
    if (std.mem.eql(u8, name, "boolean")) return .boolean;
    if (std.mem.eql(u8, name, "json")) return .json;
    return null;
}

test "database management lifecycle" {
    const allocator = std.testing.allocator;
    var mgr = DatabaseManager.init(allocator);
    defer mgr.deinit();

    const db = try mgr.createDatabase("kogi", "primary");
    const table = try mgr.createTable(db, "contacts", "directory records");
    try mgr.addTableColumn(table, "name", .text, false, "");
    try mgr.addTableColumn(table, "category", .text, true, "");

    const rec = try mgr.createRecord(db, table);
    try mgr.setRecordField(rec, "name", "Alice");
    try mgr.setRecordField(rec, "category", "client");

    {
        var matches = try mgr.queryRecordsByField(table, "category", "client", allocator);
        defer matches.deinit();
        try std.testing.expectEqual(@as(usize, 1), matches.items.len);
    }

    try mgr.deleteRecord(rec);
    {
        var matches = try mgr.queryRecordsByField(table, "category", "client", allocator);
        defer matches.deinit();
        try std.testing.expectEqual(@as(usize, 0), matches.items.len);
    }
    try std.testing.expect(mgr.getTransactions().len >= 3);
}
