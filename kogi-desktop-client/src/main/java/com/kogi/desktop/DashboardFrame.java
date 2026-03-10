package com.kogi.desktop;

import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.FlowLayout;
import java.awt.Font;
import java.awt.GridLayout;
import java.io.IOException;
import java.util.List;
import javax.swing.BorderFactory;
import javax.swing.DefaultListModel;
import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JList;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.JSplitPane;
import javax.swing.JTextArea;
import javax.swing.ListSelectionModel;
import javax.swing.SwingConstants;

public final class DashboardFrame extends JFrame {
    private static final Color BG = new Color(11, 17, 37);
    private static final Color PANEL = new Color(17, 28, 59);
    private static final Color PANEL_ALT = new Color(13, 21, 45);
    private static final Color FG = new Color(219, 231, 255);

    private final KogiApiClient api;
    private UnifiedScreenCatalog catalog;
    private final List<UnifiedScreenView> officeViews;

    private final JComboBox<String> profileSelect;
    private final JComboBox<String> modeSelect;
    private final JComboBox<String> kindSelect;
    private final DefaultListModel<UnifiedScreenView> navModel;
    private final JList<UnifiedScreenView> navList;
    private final JLabel titleLabel;
    private final JLabel subtitleLabel;
    private final JLabel seriesLabel;
    private final JLabel versionLabel;
    private final JTextArea sectionsArea;
    private final JTextArea stepsArea;
    private final JTextArea tagsArea;
    private final JTextArea sourcesArea;
    private final JTextArea output;

    public DashboardFrame(KogiApiClient api) {
        super("Kogi Desktop Client");
        this.api = api;
        this.catalog = UnifiedScreenCatalog.fallback();
        this.officeViews = buildOfficeViews();

        this.profileSelect = new JComboBox<>(new String[] {
            "Personal Profile",
            "Work Profile",
            "Business Profile",
            "Community Profile",
        });
        this.modeSelect = new JComboBox<>(new String[] {"Office Views", "Unified Views"});
        this.kindSelect = new JComboBox<>(new String[] {"Module Views", "Workflow Views"});
        this.navModel = new DefaultListModel<>();
        this.navList = new JList<>(navModel);
        this.titleLabel = new JLabel("Kogi Office");
        this.subtitleLabel = new JLabel("Module application and service views");
        this.seriesLabel = new JLabel();
        this.versionLabel = new JLabel();
        this.sectionsArea = createReadOnlyArea();
        this.stepsArea = createReadOnlyArea();
        this.tagsArea = createReadOnlyArea();
        this.sourcesArea = createReadOnlyArea();
        this.output = createReadOnlyArea();

        setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        setSize(new Dimension(1320, 810));
        setLocationRelativeTo(null);

        JPanel root = new JPanel(new BorderLayout());
        root.setBackground(BG);
        root.setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));

        JSplitPane split = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT, buildSidebar(), buildMainContent());
        split.setResizeWeight(0.26);
        split.setDividerLocation(330);
        split.setBorder(null);

        root.add(split, BorderLayout.CENTER);
        setContentPane(root);

        modeSelect.addActionListener(e -> {
            kindSelect.setEnabled(!isOfficeMode());
            reloadNavigation(currentSelectionId());
        });
        kindSelect.addActionListener(e -> {
            if (!isOfficeMode()) {
                reloadNavigation(currentSelectionId());
            }
        });
        navList.addListSelectionListener(e -> {
            if (!e.getValueIsAdjusting()) {
                renderSelectedScreen();
            }
        });

        reloadNavigation("dashboard");
        append("SYSTEM", "Desktop UI initialized in Office mode with unified fallback catalog available.");
        fetchActiveOfficeView();
    }

    private JPanel buildSidebar() {
        JPanel sidebar = new JPanel(new BorderLayout(8, 8));
        sidebar.setBackground(PANEL_ALT);
        sidebar.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(40, 62, 110)),
            BorderFactory.createEmptyBorder(12, 12, 12, 12)
        ));

        JLabel brand = new JLabel("KOGI", SwingConstants.LEFT);
        brand.setForeground(new Color(123, 167, 255));
        brand.setFont(new Font("Serif", Font.BOLD, 28));

        JPanel controls = new JPanel(new GridLayout(0, 1, 0, 6));
        controls.setOpaque(false);
        controls.add(label("Identity Profile"));
        controls.add(profileSelect);
        controls.add(label("Application Mode"));
        controls.add(modeSelect);
        controls.add(label("Unified Kind"));
        controls.add(kindSelect);

        JButton refresh = new JButton("Refresh Active View");
        refresh.addActionListener(e -> refreshActiveMode());
        controls.add(refresh);

        navList.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
        navList.setBackground(new Color(12, 20, 43));
        navList.setForeground(FG);
        navList.setSelectionBackground(new Color(35, 64, 128));
        navList.setFixedCellHeight(28);

        JPanel top = new JPanel(new BorderLayout(0, 10));
        top.setOpaque(false);
        top.add(brand, BorderLayout.NORTH);
        top.add(controls, BorderLayout.CENTER);

        sidebar.add(top, BorderLayout.NORTH);
        sidebar.add(new JScrollPane(navList), BorderLayout.CENTER);
        return sidebar;
    }

    private JPanel buildMainContent() {
        JPanel content = new JPanel(new BorderLayout(10, 10));
        content.setBackground(BG);

        JPanel header = new JPanel(new BorderLayout(8, 8));
        header.setBackground(PANEL);
        header.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(39, 60, 108)),
            BorderFactory.createEmptyBorder(12, 14, 12, 14)
        ));

        titleLabel.setForeground(FG);
        titleLabel.setFont(new Font("Serif", Font.BOLD, 30));
        subtitleLabel.setForeground(new Color(170, 191, 239));

        JPanel headerText = new JPanel(new BorderLayout());
        headerText.setOpaque(false);
        headerText.add(titleLabel, BorderLayout.NORTH);
        headerText.add(subtitleLabel, BorderLayout.CENTER);

        JPanel meta = new JPanel(new GridLayout(0, 1, 0, 4));
        meta.setOpaque(false);
        seriesLabel.setForeground(new Color(150, 199, 255));
        versionLabel.setForeground(new Color(150, 199, 255));
        meta.add(seriesLabel);
        meta.add(versionLabel);

        header.add(headerText, BorderLayout.CENTER);
        header.add(meta, BorderLayout.EAST);

        JPanel cards = new JPanel(new GridLayout(2, 2, 10, 10));
        cards.setOpaque(false);
        cards.add(panelCard("Sections", sectionsArea));
        cards.add(panelCard("Flows", stepsArea));
        cards.add(panelCard("Tags and Integrations", tagsArea));
        cards.add(panelCard("Sources and Links", sourcesArea));

        JPanel dataControls = new JPanel(new FlowLayout(FlowLayout.LEFT, 8, 0));
        dataControls.setOpaque(false);
        dataControls.add(button("Health", () -> api.health(), "HEALTH"));
        dataControls.add(button("System", () -> api.systemSummary(), "SYSTEM"));
        dataControls.add(button("Modules", () -> api.modules(), "MODULES"));
        dataControls.add(button("IMS Identities", () -> api.identities(), "IMS_IDENTITIES"));
        dataControls.add(button("IMS Profiles", () -> api.profiles(), "IMS_PROFILES"));
        dataControls.add(button("Isolation", () -> api.moduleIsolation(), "ISOLATION"));
        dataControls.add(button("Office Overview", () -> api.officeOverview(), "OFFICE_OVERVIEW"));
        dataControls.add(button("Office View", this::activeOfficeViewPayload, "OFFICE_VIEW"));

        JPanel outputPanel = panelCard("Diagnostics", output);
        outputPanel.setPreferredSize(new Dimension(0, 220));

        JPanel center = new JPanel(new BorderLayout(10, 10));
        center.setOpaque(false);
        center.add(cards, BorderLayout.CENTER);
        center.add(dataControls, BorderLayout.NORTH);

        content.add(header, BorderLayout.NORTH);
        content.add(center, BorderLayout.CENTER);
        content.add(outputPanel, BorderLayout.SOUTH);
        return content;
    }

    private JPanel panelCard(String title, JTextArea body) {
        JPanel panel = new JPanel(new BorderLayout());
        panel.setBackground(PANEL);
        panel.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(39, 60, 108)),
            BorderFactory.createEmptyBorder(10, 10, 10, 10)
        ));

        JLabel header = new JLabel(title);
        header.setForeground(new Color(187, 209, 255));
        header.setFont(header.getFont().deriveFont(Font.BOLD));

        panel.add(header, BorderLayout.NORTH);
        panel.add(new JScrollPane(body), BorderLayout.CENTER);
        return panel;
    }

    private JButton button(String label, ApiCall call, String context) {
        JButton button = new JButton(label);
        button.addActionListener(e -> runCall(context, call));
        return button;
    }

    private JLabel label(String text) {
        JLabel label = new JLabel(text);
        label.setForeground(new Color(163, 189, 246));
        label.setFont(label.getFont().deriveFont(Font.BOLD, 11f));
        return label;
    }

    private JTextArea createReadOnlyArea() {
        JTextArea area = new JTextArea();
        area.setEditable(false);
        area.setLineWrap(true);
        area.setWrapStyleWord(true);
        area.setBackground(new Color(8, 14, 32));
        area.setForeground(FG);
        area.setFont(new Font("Monospaced", Font.PLAIN, 12));
        return area;
    }

    private boolean isOfficeMode() {
        return modeSelect.getSelectedIndex() == 0;
    }

    private String selectedUnifiedKind() {
        return kindSelect.getSelectedIndex() == 1 ? "workflow" : "module";
    }

    private String currentSelectionId() {
        UnifiedScreenView selected = navList.getSelectedValue();
        return selected == null ? "" : selected.id();
    }

    private void reloadNavigation(String preserveId) {
        navModel.clear();
        List<UnifiedScreenView> screens = isOfficeMode()
            ? officeViews
            : catalog.screensForKind(selectedUnifiedKind());

        for (UnifiedScreenView screen : screens) {
            navModel.addElement(screen);
        }

        if (!preserveId.isBlank()) {
            for (int i = 0; i < navModel.size(); i++) {
                if (preserveId.equals(navModel.get(i).id())) {
                    navList.setSelectedIndex(i);
                    return;
                }
            }
        }

        if (!navModel.isEmpty()) {
            navList.setSelectedIndex(0);
        } else {
            renderEmpty();
        }
    }

    private void renderSelectedScreen() {
        UnifiedScreenView selected = navList.getSelectedValue();
        if (selected == null) {
            renderEmpty();
            return;
        }

        titleLabel.setText(selected.title());
        if (isOfficeMode()) {
            subtitleLabel.setText("Kogi Office application view from module service payload.");
            seriesLabel.setText("Series: kogi-office-application");
            versionLabel.setText("Version: v0.1.0");
            sourcesArea.setText(officeSources(selected.id()));
            fetchActiveOfficeView();
        } else if ("workflow".equals(selected.kind())) {
            subtitleLabel.setText("Unified workflow view linked to module: " + safeModule(selected.module()));
            seriesLabel.setText("Series: " + catalog.series());
            versionLabel.setText("Version: " + catalog.version());
            sourcesArea.setText(lines(catalog.sources(), "(no source files)"));
        } else {
            subtitleLabel.setText("Unified module view reconciled from screen-flow documents");
            seriesLabel.setText("Series: " + catalog.series());
            versionLabel.setText("Version: " + catalog.version());
            sourcesArea.setText(lines(catalog.sources(), "(no source files)"));
        }

        sectionsArea.setText(lines(selected.sections(), "(no sections for this view)"));
        stepsArea.setText(lines(selected.steps(), "(no flow steps for this view)"));
        tagsArea.setText(tagsText(selected));
    }

    private String tagsText(UnifiedScreenView view) {
        StringBuilder builder = new StringBuilder();
        builder.append(lines(view.tags(), "(no tags)"));
        if (!view.module().isBlank()) {
            builder.append("\n\nLinked module: ").append(view.module());
        }
        return builder.toString();
    }

    private String officeSources(String viewId) {
        return switch (viewId) {
            case "dashboard" -> "- /api/v1/office/dashboard\n- /api/v1/office";
            case "portfolio" -> "- /api/v1/office/portfolio\n- /api/v1/office";
            case "timeline" -> "- /api/v1/office/timeline\n- /api/v1/office";
            case "workspace" -> "- /api/v1/office/workspace\n- /api/v1/office";
            case "assistant" -> "- /api/v1/office/assistant\n- /api/v1/office";
            default -> "- /api/v1/office";
        };
    }

    private String safeModule(String module) {
        return module == null || module.isBlank() ? "(none)" : module;
    }

    private static String lines(List<String> values, String whenEmpty) {
        if (values == null || values.isEmpty()) {
            return whenEmpty;
        }
        StringBuilder builder = new StringBuilder();
        for (String value : values) {
            builder.append("- ").append(value).append('\n');
        }
        return builder.toString().trim();
    }

    private void renderEmpty() {
        titleLabel.setText("No View Selected");
        subtitleLabel.setText("Select a module or workflow from the navigator");
        seriesLabel.setText("");
        versionLabel.setText("");
        sectionsArea.setText("");
        stepsArea.setText("");
        tagsArea.setText("");
        sourcesArea.setText("");
    }

    private void refreshActiveMode() {
        if (isOfficeMode()) {
            fetchActiveOfficeView();
            return;
        }
        syncCatalog();
    }

    private void syncCatalog() {
        String preserveId = currentSelectionId();
        try {
            String flat = api.unifiedScreensFlat();
            catalog = catalog.mergeFlat(flat);
            reloadNavigation(preserveId);
            append("CATALOG", "Unified screen catalog synced from /api/v1/screens/unified/flat");
        } catch (IOException | InterruptedException ex) {
            append("CATALOG_ERROR", ex.getMessage());
        }
    }

    private void fetchActiveOfficeView() {
        if (!isOfficeMode()) {
            return;
        }
        runCall("OFFICE_VIEW", this::activeOfficeViewPayload);
    }

    private String activeOfficeViewPayload() throws IOException, InterruptedException {
        UnifiedScreenView selected = navList.getSelectedValue();
        String id = selected == null ? "dashboard" : selected.id();
        return switch (id) {
            case "dashboard" -> api.officeDashboard();
            case "portfolio" -> api.officePortfolio();
            case "timeline" -> api.officeTimeline();
            case "workspace" -> api.officeWorkspace();
            case "assistant" -> api.officeAssistant();
            default -> api.officeOverview();
        };
    }

    private void runCall(String context, ApiCall call) {
        try {
            append(context, call.run());
        } catch (IOException | InterruptedException ex) {
            append(context + "_ERROR", ex.getMessage());
        }
    }

    private void append(String context, String payload) {
        output.append("[" + context + "]\n" + payload + "\n\n");
        output.setCaretPosition(output.getDocument().getLength());
    }

    private static List<UnifiedScreenView> buildOfficeViews() {
        return List.of(
            new UnifiedScreenView(
                "office",
                "dashboard",
                "Office Dashboard",
                "kogi.office",
                List.of("projects", "programs", "notifications", "messages", "feeds", "personas"),
                List.of(
                    "Active projects and programs",
                    "Portfolio attention and notifications",
                    "Direct messages",
                    "Event/community/marketplace/exchange feed",
                    "Personas and roles",
                    "Quick links and access"
                ),
                List.of(
                    "Review attention queue",
                    "Resolve notifications",
                    "Open quick links into workspace and integrations"
                )
            ),
            new UnifiedScreenView(
                "office",
                "portfolio",
                "Office Portfolio",
                "kogi.office",
                List.of("tiled", "tree", "modular-grid", "focus-view", "metadata", "version-control"),
                List.of(
                    "Tiled/tree/modular grid views",
                    "Item types: projects/programs/resources/assets/capital/investments/solutions/documents/misc/custom",
                    "Focus view with detail pane",
                    "Item containers: binder/book/notebook/playbook/folders/files/version control/metadata"
                ),
                List.of(
                    "Select grid mode",
                    "Choose focus item",
                    "Open item container stack"
                )
            ),
            new UnifiedScreenView(
                "office",
                "timeline",
                "Office Timeline",
                "kogi.office",
                List.of("calendars", "schedules", "roadmaps", "gantts", "personal-timelines"),
                List.of(
                    "Calendars and schedules",
                    "Roadmap timelines",
                    "Gantt planning and critical path",
                    "Personal timeline overlays"
                ),
                List.of(
                    "Plan schedule windows",
                    "Sync roadmap milestones",
                    "Track gantt critical path"
                )
            ),
            new UnifiedScreenView(
                "office",
                "workspace",
                "Office Workspace",
                "kogi.office",
                List.of("operations", "tactics", "strategy", "governance", "stories", "toolchains"),
                List.of(
                    "Personal work/operations/tactics/strategy/governance board",
                    "User stories and work packages",
                    "Personal content management system",
                    "Tools, toolchains, toolkits, toolsets, and tool links"
                ),
                List.of(
                    "Prioritize stories",
                    "Execute work packages",
                    "Launch linked toolchain"
                )
            ),
            new UnifiedScreenView(
                "office",
                "assistant",
                "Office Assistant",
                "kogi.office",
                List.of("ai", "chat", "context-window", "discover", "recommendations", "subscriptions"),
                List.of(
                    "Digital AI assistant and chat",
                    "Context window and memory sources",
                    "Discover/recommendations/subscriptions",
                    "Explore and for-you streams"
                ),
                List.of(
                    "Ask contextual question",
                    "Apply recommendation",
                    "Subscribe to recurring insights"
                )
            )
        );
    }

    @FunctionalInterface
    private interface ApiCall {
        String run() throws IOException, InterruptedException;
    }
}
