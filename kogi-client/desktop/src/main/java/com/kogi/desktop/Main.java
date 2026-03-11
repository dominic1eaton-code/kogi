package com.kogi.desktop;

import javax.swing.SwingUtilities;

public final class Main {
    private Main() {}

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> {
            DashboardFrame frame = new DashboardFrame(new KogiApiClient("http://127.0.0.1:8080"));
            frame.setVisible(true);
        });
    }
}