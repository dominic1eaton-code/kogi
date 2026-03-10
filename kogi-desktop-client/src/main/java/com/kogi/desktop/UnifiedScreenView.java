package com.kogi.desktop;

import java.util.List;
import java.util.Objects;

public final class UnifiedScreenView {
    private final String kind;
    private final String id;
    private final String title;
    private final String module;
    private final List<String> tags;
    private final List<String> sections;
    private final List<String> steps;

    public UnifiedScreenView(
        String kind,
        String id,
        String title,
        String module,
        List<String> tags,
        List<String> sections,
        List<String> steps
    ) {
        this.kind = kind;
        this.id = id;
        this.title = title;
        this.module = module;
        this.tags = List.copyOf(tags);
        this.sections = List.copyOf(sections);
        this.steps = List.copyOf(steps);
    }

    public static UnifiedScreenView module(
        String id,
        String title,
        List<String> tags,
        List<String> sections
    ) {
        return new UnifiedScreenView("module", id, title, "", tags, sections, List.of());
    }

    public static UnifiedScreenView workflow(
        String id,
        String title,
        String module,
        List<String> tags,
        List<String> steps
    ) {
        return new UnifiedScreenView("workflow", id, title, module, tags, List.of(), steps);
    }

    public UnifiedScreenView withTitle(String newTitle) {
        return new UnifiedScreenView(kind, id, newTitle, module, tags, sections, steps);
    }

    public String kind() {
        return kind;
    }

    public String id() {
        return id;
    }

    public String title() {
        return title;
    }

    public String module() {
        return module;
    }

    public List<String> tags() {
        return tags;
    }

    public List<String> sections() {
        return sections;
    }

    public List<String> steps() {
        return steps;
    }

    @Override
    public String toString() {
        return title;
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (!(obj instanceof UnifiedScreenView other)) {
            return false;
        }
        return Objects.equals(kind, other.kind) && Objects.equals(id, other.id);
    }

    @Override
    public int hashCode() {
        return Objects.hash(kind, id);
    }
}
