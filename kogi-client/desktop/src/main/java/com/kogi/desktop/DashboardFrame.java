package com.kogi.desktop;

import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.FlowLayout;
import java.awt.Font;
import java.awt.GridLayout;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import javax.swing.BorderFactory;
import javax.swing.BoxLayout;
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
import javax.swing.JTextField;
import javax.swing.ListSelectionModel;
import javax.swing.SwingConstants;
import javax.swing.event.DocumentEvent;
import javax.swing.event.DocumentListener;

public final class DashboardFrame extends JFrame {
    private static final Color BG = new Color(7, 11, 22);
    private static final Color PANEL = new Color(12, 26, 62);
    private static final Color PANEL_SOFT = new Color(10, 20, 48);
    private static final Color PANEL_RICH = new Color(15, 42, 94);
    private static final Color FG = new Color(220, 236, 255);
    private static final Color MUTED = new Color(148, 181, 236);
    private static final Color ACCENT = new Color(95, 225, 255);

    private final KogiApiClient api;
    private UnifiedScreenCatalog catalog;
    private final List<UnifiedScreenView> officeViews;
    private final List<UnifiedScreenView> platformViews;
    private final List<UnifiedScreenView> providerViews;
    private List<UnifiedScreenView> currentViews;

    private final JComboBox<String> profileSelect;
    private final JComboBox<String> modeSelect;
    private final JComboBox<String> kindSelect;
    private final JTextField searchField;
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
        this.platformViews = buildPlatformViews();
        this.providerViews = buildProviderViews();
        this.currentViews = new ArrayList<>(officeViews);

        this.profileSelect = new JComboBox<>(new String[] {
            "Personal Profile",
            "Work Profile",
            "Business Profile",
            "Community Profile",
        });
        this.modeSelect = new JComboBox<>(new String[] {"Office Views", "Platform Views", "Provider Registry", "Unified Views"});
        this.kindSelect = new JComboBox<>(new String[] {"Module Views", "Workflow Views"});
        this.searchField = new JTextField();
        this.navModel = new DefaultListModel<>();
        this.navList = new JList<>(navModel);
        this.titleLabel = new JLabel("Kogi Office");
        this.subtitleLabel = new JLabel("Modern office control center");
        this.seriesLabel = new JLabel();
        this.versionLabel = new JLabel();
        this.sectionsArea = createReadOnlyArea();
        this.stepsArea = createReadOnlyArea();
        this.tagsArea = createReadOnlyArea();
        this.sourcesArea = createReadOnlyArea();
        this.output = createReadOnlyArea();

        setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        setSize(new Dimension(1380, 860));
        setLocationRelativeTo(null);

        JPanel root = new JPanel(new BorderLayout(10, 10));
        root.setBackground(BG);
        root.setBorder(BorderFactory.createEmptyBorder(10, 10, 10, 10));
        root.add(buildTopBar(), BorderLayout.NORTH);

        JSplitPane shellSplit = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT, buildNavigationShell(), buildMainContent());
        shellSplit.setDividerLocation(380);
        shellSplit.setResizeWeight(0.28);
        shellSplit.setBorder(null);
        root.add(shellSplit, BorderLayout.CENTER);

        setContentPane(root);
        kindSelect.setEnabled(isUnifiedMode());

        modeSelect.addActionListener(e -> {
            kindSelect.setEnabled(isUnifiedMode());
            reloadNavigation(currentSelectionId());
        });
        kindSelect.addActionListener(e -> {
            if (isUnifiedMode()) {
                reloadNavigation(currentSelectionId());
            }
        });
        navList.addListSelectionListener(e -> {
            if (!e.getValueIsAdjusting()) {
                renderSelectedScreen();
            }
        });
        searchField.getDocument().addDocumentListener(new DocumentListener() {
            @Override
            public void insertUpdate(DocumentEvent e) {
                applyNavFilter();
            }

            @Override
            public void removeUpdate(DocumentEvent e) {
                applyNavFilter();
            }

            @Override
            public void changedUpdate(DocumentEvent e) {
                applyNavFilter();
            }
        });

        reloadNavigation("dashboard");
        append("SYSTEM", "Desktop UI initialized with modern office shell and live module diagnostics.");
        fetchActiveOfficeView();
    }

    private JPanel buildTopBar() {
        JPanel top = new JPanel(new BorderLayout(10, 10));
        top.setBackground(PANEL_SOFT);
        top.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(53, 98, 182)),
            BorderFactory.createEmptyBorder(10, 12, 10, 12)
        ));

        JLabel brand = new JLabel("KOGI OS", SwingConstants.LEFT);
        brand.setForeground(ACCENT);
        brand.setFont(new Font("SansSerif", Font.BOLD, 24));

        styleSearchField(searchField);
        searchField.setToolTipText("Filter modules/workflows");
        searchField.setText("");

        JPanel right = new JPanel(new FlowLayout(FlowLayout.RIGHT, 8, 0));
        right.setOpaque(false);
        JButton refresh = actionButton("Refresh", this::refreshActiveMode);
        styleCombo(profileSelect);
        right.add(refresh);
        right.add(profileSelect);

        top.add(brand, BorderLayout.WEST);
        top.add(searchField, BorderLayout.CENTER);
        top.add(right, BorderLayout.EAST);
        return top;
    }

    private JPanel buildNavigationShell() {
        JPanel shell = new JPanel(new BorderLayout(8, 8));
        shell.setOpaque(false);

        JPanel rail = new JPanel(new GridLayout(0, 1, 0, 8));
        rail.setPreferredSize(new Dimension(58, 0));
        rail.setBackground(PANEL_SOFT);
        rail.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(43, 83, 156)),
            BorderFactory.createEmptyBorder(10, 8, 10, 8)
        ));
        rail.add(iconButton("◎"));
        rail.add(iconButton("▦"));
        rail.add(iconButton("↺"));
        rail.add(iconButton("⚙"));

        JPanel sidebar = new JPanel(new BorderLayout(8, 8));
        sidebar.setBackground(PANEL_SOFT);
        sidebar.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(43, 83, 156)),
            BorderFactory.createEmptyBorder(10, 10, 10, 10)
        ));

        JPanel controls = new JPanel(new GridLayout(0, 1, 0, 8));
        controls.setOpaque(false);
        controls.add(label("Application Mode"));
        styleCombo(modeSelect);
        controls.add(modeSelect);
        controls.add(label("Unified Kind"));
        styleCombo(kindSelect);
        controls.add(kindSelect);

        navList.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
        navList.setBackground(new Color(8, 19, 46));
        navList.setForeground(FG);
        navList.setSelectionBackground(new Color(37, 88, 173));
        navList.setFixedCellHeight(30);
        navList.setBorder(BorderFactory.createEmptyBorder(6, 6, 6, 6));

        sidebar.add(controls, BorderLayout.NORTH);
        sidebar.add(new JScrollPane(navList), BorderLayout.CENTER);

        shell.add(rail, BorderLayout.WEST);
        shell.add(sidebar, BorderLayout.CENTER);
        return shell;
    }

    private JPanel buildMainContent() {
        JPanel content = new JPanel(new BorderLayout(10, 10));
        content.setBackground(BG);

        JPanel header = new JPanel(new BorderLayout(8, 8));
        header.setBackground(PANEL_RICH);
        header.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(51, 92, 168)),
            BorderFactory.createEmptyBorder(12, 14, 12, 14)
        ));

        titleLabel.setForeground(FG);
        titleLabel.setFont(new Font("SansSerif", Font.BOLD, 30));
        subtitleLabel.setForeground(MUTED);

        JPanel headerText = new JPanel(new GridLayout(0, 1));
        headerText.setOpaque(false);
        headerText.add(titleLabel);
        headerText.add(subtitleLabel);

        JPanel meta = new JPanel(new GridLayout(0, 1, 0, 4));
        meta.setOpaque(false);
        seriesLabel.setForeground(ACCENT);
        versionLabel.setForeground(ACCENT);
        meta.add(seriesLabel);
        meta.add(versionLabel);

        header.add(headerText, BorderLayout.CENTER);
        header.add(meta, BorderLayout.EAST);

        JPanel cards = new JPanel(new GridLayout(2, 2, 10, 10));
        cards.setOpaque(false);
        cards.add(panelCard("Sections", sectionsArea));
        cards.add(panelCard("Flows", stepsArea));
        cards.add(panelCard("Integrations and Tags", tagsArea));
        cards.add(panelCard("Sources and Links", sourcesArea));

        JPanel toolRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 8, 0));
        toolRow.setOpaque(false);
        toolRow.add(actionButton("Health", () -> runCall("HEALTH", api::health)));
        toolRow.add(actionButton("System", () -> runCall("SYSTEM", api::systemSummary)));
        toolRow.add(actionButton("Host", () -> runCall("HOST", api::hostSummary)));
        toolRow.add(actionButton("Host Components", () -> runCall("HOST_COMPONENTS", api::hostComponents)));
        toolRow.add(actionButton("Modules", () -> runCall("MODULES", api::modules)));
        toolRow.add(actionButton("Engine", () -> runCall("ENGINE", api::engineOverview)));
        toolRow.add(actionButton("Engine Runtime", () -> runCall("ENGINE_RUNTIME", api::engineRuntime)));
        toolRow.add(actionButton("Database", () -> runCall("DATABASE", api::databaseRuntime)));
        toolRow.add(actionButton("Providers", () -> runCall("PROVIDERS", api::providersSnapshot)));
        toolRow.add(actionButton("Affiliates", () -> runCall("AFFILIATES", api::providersAffiliates)));
        toolRow.add(actionButton("IMS Identities", () -> runCall("IMS_IDENTITIES", api::identities)));
        toolRow.add(actionButton("IMS Profiles", () -> runCall("IMS_PROFILES", api::profiles)));
        toolRow.add(actionButton("Isolation", () -> runCall("ISOLATION", api::moduleIsolation)));
        toolRow.add(actionButton("Office Overview", () -> runCall("OFFICE_OVERVIEW", api::officeOverview)));
        toolRow.add(actionButton("Office View", this::fetchActiveOfficeView));
        toolRow.add(actionButton("Platform View", this::fetchActivePlatformView));
        toolRow.add(actionButton("Provider View", this::fetchActiveProviderView));

        JPanel outputPanel = panelCard("Realtime Diagnostics", output);
        outputPanel.setPreferredSize(new Dimension(0, 250));

        JPanel center = new JPanel(new BorderLayout(10, 10));
        center.setOpaque(false);
        center.add(toolRow, BorderLayout.NORTH);
        center.add(cards, BorderLayout.CENTER);

        content.add(header, BorderLayout.NORTH);
        content.add(center, BorderLayout.CENTER);
        content.add(outputPanel, BorderLayout.SOUTH);
        return content;
    }

    private JPanel panelCard(String title, JTextArea body) {
        JPanel panel = new JPanel(new BorderLayout(8, 8));
        panel.setBackground(PANEL);
        panel.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(45, 84, 156)),
            BorderFactory.createEmptyBorder(10, 10, 10, 10)
        ));

        JLabel heading = new JLabel(title);
        heading.setForeground(new Color(186, 215, 255));
        heading.setFont(new Font("SansSerif", Font.BOLD, 13));
        panel.add(heading, BorderLayout.NORTH);
        panel.add(new JScrollPane(body), BorderLayout.CENTER);
        return panel;
    }

    private JButton actionButton(String text, Runnable action) {
        JButton button = new JButton(text);
        button.setBackground(new Color(16, 42, 91));
        button.setForeground(new Color(198, 222, 255));
        button.setFocusPainted(false);
        button.setBorder(BorderFactory.createLineBorder(new Color(66, 110, 189)));
        button.addActionListener(e -> action.run());
        return button;
    }

    private JButton iconButton(String icon) {
        JButton button = new JButton(icon);
        button.setBackground(new Color(16, 40, 82));
        button.setForeground(new Color(144, 199, 255));
        button.setFocusPainted(false);
        button.setBorder(BorderFactory.createLineBorder(new Color(59, 100, 177)));
        return button;
    }

    private JLabel label(String text) {
        JLabel label = new JLabel(text);
        label.setForeground(MUTED);
        label.setFont(new Font("SansSerif", Font.BOLD, 11));
        return label;
    }

    private JTextArea createReadOnlyArea() {
        JTextArea area = new JTextArea();
        area.setEditable(false);
        area.setLineWrap(true);
        area.setWrapStyleWord(true);
        area.setBackground(new Color(7, 16, 37));
        area.setForeground(FG);
        area.setFont(new Font("Monospaced", Font.PLAIN, 12));
        return area;
    }

    private void styleCombo(JComboBox<String> combo) {
        combo.setBackground(new Color(9, 28, 63));
        combo.setForeground(FG);
    }

    private void styleSearchField(JTextField field) {
        field.setBackground(new Color(8, 20, 48));
        field.setForeground(FG);
        field.setCaretColor(FG);
        field.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(new Color(65, 105, 178)),
            BorderFactory.createEmptyBorder(8, 10, 8, 10)
        ));
    }

    private boolean isOfficeMode() {
        return modeSelect.getSelectedIndex() == 0;
    }

    private boolean isPlatformMode() {
        return modeSelect.getSelectedIndex() == 1;
    }

    private boolean isProviderMode() {
        return modeSelect.getSelectedIndex() == 2;
    }

    private boolean isUnifiedMode() {
        return modeSelect.getSelectedIndex() == 3;
    }

    private String selectedUnifiedKind() {
        return kindSelect.getSelectedIndex() == 1 ? "workflow" : "module";
    }

    private String currentSelectionId() {
        UnifiedScreenView selected = navList.getSelectedValue();
        return selected == null ? "" : selected.id();
    }

    private void reloadNavigation(String preserveId) {
        List<UnifiedScreenView> views;
        if (isOfficeMode()) {
            views = officeViews;
        } else if (isPlatformMode()) {
            views = platformViews;
        } else if (isProviderMode()) {
            views = providerViews;
        } else {
            views = catalog.screensForKind(selectedUnifiedKind());
        }
        currentViews = new ArrayList<>(views);
        applyNavFilter(preserveId);
    }

    private void applyNavFilter() {
        applyNavFilter(currentSelectionId());
    }

    private void applyNavFilter(String preserveId) {
        String query = searchField.getText() == null ? "" : searchField.getText().trim().toLowerCase();

        navModel.clear();
        List<UnifiedScreenView> filtered = new ArrayList<>();
        for (UnifiedScreenView view : currentViews) {
            if (query.isEmpty() || view.title().toLowerCase().contains(query)) {
                filtered.add(view);
                navModel.addElement(view);
            }
        }

        if (!preserveId.isBlank()) {
            for (int i = 0; i < navModel.size(); i++) {
                if (preserveId.equals(navModel.get(i).id())) {
                    navList.setSelectedIndex(i);
                    return;
                }
            }
        }

        if (!filtered.isEmpty()) {
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
            versionLabel.setText("Version: v0.3.0");
            sourcesArea.setText(officeSources(selected.id()));
            fetchActiveOfficeView();
        } else if (isPlatformMode()) {
            subtitleLabel.setText("Platform runtime view stitched from host, engine, and database services.");
            seriesLabel.setText("Series: kogi-platform-architecture");
            versionLabel.setText("Version: v1.0.0");
            sourcesArea.setText(platformSources(selected.id()));
            fetchActivePlatformView();
        } else if (isProviderMode()) {
            subtitleLabel.setText("Provider registry view across platforms, providers, and affiliates.");
            seriesLabel.setText("Series: kogi-provider-registry");
            versionLabel.setText("Version: v0.2.0");
            sourcesArea.setText(providerSources(selected.id()));
            fetchActiveProviderView();
        } else if ("workflow".equals(selected.kind())) {
            subtitleLabel.setText("Unified workflow view linked to module: " + safeModule(selected.module()));
            seriesLabel.setText("Series: " + catalog.series());
            versionLabel.setText("Version: " + catalog.version());
            sourcesArea.setText(lines(catalog.sources(), "(no source files)"));
        } else {
            subtitleLabel.setText("Unified module view reconciled from screen-flow documents.");
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

    private String platformSources(String viewId) {
        return switch (viewId) {
            case "system" -> "- /api/v1/system\n- /api/v1/host\n- /api/v1/modules";
            case "host" -> "- /api/v1/host\n- /api/v1/host/components";
            case "host_components" -> "- /api/v1/host/components";
            case "engine" -> "- /api/v1/engine/system\n- /api/v1/engine/runtime";
            case "engine_runtime" -> "- /api/v1/engine/runtime";
            case "database" -> "- /api/v1/database/runtime\n- /api/v1/database/query";
            case "modules" -> "- /api/v1/modules";
            case "autonomy" -> "- /api/v1/autonomy/capabilities";
            default -> "- /api/v1/system";
        };
    }

    private String providerSources(String viewId) {
        return switch (viewId) {
            case "snapshot" -> "- /api/v1/providers";
            case "platforms" -> "- /api/v1/providers/platforms";
            case "providers" -> "- /api/v1/providers/providers";
            case "resources" -> "- /api/v1/providers/resources";
            case "versions" -> "- /api/v1/providers/versions";
            case "metadata" -> "- /api/v1/providers/metadata";
            case "data" -> "- /api/v1/providers/data";
            case "affiliates" -> "- /api/v1/providers/affiliates";
            case "affiliate_links" -> "- /api/v1/providers/affiliate-links";
            default -> "- /api/v1/providers";
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
        if (isPlatformMode()) {
            fetchActivePlatformView();
            return;
        }
        if (isProviderMode()) {
            fetchActiveProviderView();
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

    private void fetchActivePlatformView() {
        if (!isPlatformMode()) {
            return;
        }
        runCall("PLATFORM_VIEW", this::activePlatformViewPayload);
    }

    private String activePlatformViewPayload() throws IOException, InterruptedException {
        UnifiedScreenView selected = navList.getSelectedValue();
        String id = selected == null ? "system" : selected.id();
        return switch (id) {
            case "system" -> api.systemSummary();
            case "host" -> api.hostSummary();
            case "host_components" -> api.hostComponents();
            case "engine" -> api.engineOverview();
            case "engine_runtime" -> api.engineRuntime();
            case "database" -> api.databaseRuntime();
            case "modules" -> api.modules();
            case "autonomy" -> api.autonomyCapabilities();
            default -> api.systemSummary();
        };
    }

    private void fetchActiveProviderView() {
        if (!isProviderMode()) {
            return;
        }
        runCall("PROVIDER_VIEW", this::activeProviderViewPayload);
    }

    private String activeProviderViewPayload() throws IOException, InterruptedException {
        UnifiedScreenView selected = navList.getSelectedValue();
        String id = selected == null ? "snapshot" : selected.id();
        return switch (id) {
            case "snapshot" -> api.providersSnapshot();
            case "platforms" -> api.providersPlatforms();
            case "providers" -> api.providersList();
            case "resources" -> api.providersResources();
            case "versions" -> api.providersVersions();
            case "metadata" -> api.providersMetadata();
            case "data" -> api.providersDataAssets();
            case "affiliates" -> api.providersAffiliates();
            case "affiliate_links" -> api.providersAffiliateLinks();
            default -> api.providersSnapshot();
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

    private static List<UnifiedScreenView> buildPlatformViews() {
        return List.of(
            new UnifiedScreenView(
                "platform",
                "system",
                "Platform System",
                "",
                List.of("kernel", "host", "modules", "registry"),
                List.of(
                    "Kernel + host mode",
                    "Module and component counts",
                    "Provider registry totals",
                    "Data flow map"
                ),
                List.of(
                    "Review active services",
                    "Confirm data flow",
                    "Audit health status"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "host",
                "Host Summary",
                "",
                List.of("host", "components", "providers"),
                List.of(
                    "Host boot status",
                    "Component and module totals",
                    "Provider registry totals"
                ),
                List.of(
                    "Inspect host runtime",
                    "Verify module orchestration"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "host_components",
                "Host Components",
                "",
                List.of("components", "limits", "network"),
                List.of(
                    "Component inventory",
                    "Resource limits",
                    "Network managers"
                ),
                List.of(
                    "Inspect component limits",
                    "Audit active component set"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "engine",
                "Engine Overview",
                "",
                List.of("engine", "data", "analytics"),
                List.of(
                    "Engine status",
                    "Capabilities",
                    "Ingest topics"
                ),
                List.of(
                    "Review ingest readiness",
                    "Validate analytics capabilities"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "engine_runtime",
                "Engine Runtime",
                "",
                List.of("engine", "runtime", "service"),
                List.of(
                    "Runtime status",
                    "Service health",
                    "Latency and throughput"
                ),
                List.of(
                    "Ping engine runtime",
                    "Verify service health"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "database",
                "Database Runtime",
                "",
                List.of("database", "storage", "query"),
                List.of(
                    "Runtime status",
                    "Query interface",
                    "Data store health"
                ),
                List.of(
                    "Run validation query",
                    "Review runtime health"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "modules",
                "Module Registry",
                "",
                List.of("modules", "capabilities", "integrations"),
                List.of(
                    "Module inventory",
                    "Capabilities",
                    "Integrations"
                ),
                List.of(
                    "Review module versions",
                    "Verify integration coverage"
                )
            ),
            new UnifiedScreenView(
                "platform",
                "autonomy",
                "Autonomy Capabilities",
                "",
                List.of("identity", "workspace", "registry"),
                List.of(
                    "Identity management",
                    "Workspace organization",
                    "Connection registry",
                    "Asset vault"
                ),
                List.of(
                    "Review autonomy coverage",
                    "Audit capability list"
                )
            )
        );
    }

    private static List<UnifiedScreenView> buildProviderViews() {
        return List.of(
            new UnifiedScreenView(
                "providers",
                "snapshot",
                "Provider Snapshot",
                "",
                List.of("registry", "totals", "affiliates"),
                List.of(
                    "Registry totals",
                    "Active providers",
                    "Active platforms",
                    "Affiliate links"
                ),
                List.of(
                    "Review registry health",
                    "Validate active inventory"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "platforms",
                "Platforms",
                "",
                List.of("platforms", "status", "tags"),
                List.of(
                    "Platform catalog",
                    "Status overview",
                    "Support contacts"
                ),
                List.of(
                    "Audit platform coverage",
                    "Verify platform status"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "providers",
                "Providers",
                "",
                List.of("providers", "owners", "versions"),
                List.of(
                    "Provider inventory",
                    "Owners and contacts",
                    "Version coverage"
                ),
                List.of(
                    "Review provider status",
                    "Check version coverage"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "resources",
                "Provider Resources",
                "",
                List.of("resources", "endpoints", "credentials"),
                List.of(
                    "Resource inventory",
                    "Environments",
                    "Credential references"
                ),
                List.of(
                    "Inspect resource endpoints",
                    "Validate credentials"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "versions",
                "Provider Versions",
                "",
                List.of("versions", "release", "compatibility"),
                List.of(
                    "Version inventory",
                    "Release status",
                    "Compatibility matrix"
                ),
                List.of(
                    "Validate version coverage",
                    "Review release notes"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "metadata",
                "Provider Metadata",
                "",
                List.of("metadata", "scopes", "policies"),
                List.of(
                    "Metadata entries",
                    "Scopes and policies",
                    "Update times"
                ),
                List.of(
                    "Audit metadata coverage",
                    "Review scopes"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "data",
                "Provider Data Assets",
                "",
                List.of("datasets", "sync", "storage"),
                List.of(
                    "Datasets",
                    "Sync status",
                    "Storage locations"
                ),
                List.of(
                    "Check sync status",
                    "Validate data coverage"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "affiliates",
                "Affiliates",
                "",
                List.of("affiliates", "partners", "contacts"),
                List.of(
                    "Affiliate inventory",
                    "Partner metadata",
                    "Contact points"
                ),
                List.of(
                    "Review affiliate status",
                    "Validate contacts"
                )
            ),
            new UnifiedScreenView(
                "providers",
                "affiliate_links",
                "Affiliate Links",
                "",
                List.of("links", "channels", "tracking"),
                List.of(
                    "Link inventory",
                    "Channels",
                    "Tracking URLs"
                ),
                List.of(
                    "Audit affiliate links",
                    "Validate tracking configuration"
                )
            )
        );
    }

    @FunctionalInterface
    private interface ApiCall {
        String run() throws IOException, InterruptedException;
    }
}
