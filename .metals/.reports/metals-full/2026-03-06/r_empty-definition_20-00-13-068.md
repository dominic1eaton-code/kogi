error id: file:///C:/dev/ws/kogi/src_claude/KogiApp.java:com/kogi/desktop/services/AuthService#
file:///C:/dev/ws/kogi/src_claude/KogiApp.java
empty definition using pc, found symbol in pc: com/kogi/desktop/services/AuthService#
empty definition using semanticdb
empty definition using fallback
non-local guesses:

offset: 159
uri: file:///C:/dev/ws/kogi/src_claude/KogiApp.java
text:
```scala
// desktop/src/main/java/com/kogi/desktop/KogiApp.java
package com.kogi.desktop;

import com.kogi.desktop.services.ApiClient;
import com.kogi.desktop.services.@@AuthService;
import com.kogi.desktop.controllers.MainController;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXMLLoader;
import javafx.scene.Scene;
import javafx.scene.image.Image;
import javafx.scene.layout.BorderPane;
import javafx.stage.Stage;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

/**
 * Kogi Desktop Client — JavaFX entry point.
 * Connects to the Kogi Go API and displays the workspace.
 */
public class KogiApp extends Application {

    private static final Logger log = LoggerFactory.getLogger(KogiApp.class);
    private static final String APP_TITLE   = "Kogi — Independent Worker OS";
    private static final String API_BASE_URL = System.getenv().getOrDefault(
        "KOGI_API_URL", "http://localhost:8080/api/v1");

    private ApiClient  apiClient;
    private AuthService authService;

    @Override
    public void init() {
        log.info("Kogi Desktop v0.1.0 starting");
        apiClient   = new ApiClient(API_BASE_URL);
        authService = new AuthService(apiClient);
    }

    @Override
    public void start(Stage primaryStage) throws IOException {
        primaryStage.setTitle(APP_TITLE);
        primaryStage.setMinWidth(1200);
        primaryStage.setMinHeight(750);

        // Load the main layout
        FXMLLoader loader = new FXMLLoader(
            getClass().getResource("/fxml/MainLayout.fxml"));

        MainController controller = new MainController(apiClient, authService);
        loader.setController(controller);

        BorderPane root = loader.load();
        Scene scene = new Scene(root, 1400, 900);
        scene.getStylesheets().add(
            getClass().getResource("/css/kogi-dark.css").toExternalForm());

        primaryStage.setScene(scene);
        primaryStage.setOnCloseRequest(e -> {
            log.info("Kogi Desktop shutting down");
            Platform.exit();
            System.exit(0);
        });

        primaryStage.show();
        log.info("Kogi Desktop launched. API: {}", API_BASE_URL);
    }

    @Override
    public void stop() {
        if (apiClient != null) apiClient.close();
        log.info("Kogi Desktop closed.");
    }

    public static void main(String[] args) {
        launch(args);
    }
}

```


#### Short summary: 

empty definition using pc, found symbol in pc: com/kogi/desktop/services/AuthService#