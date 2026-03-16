app.config(function($routeProvider) {

$routeProvider

.when("/login", {
    templateUrl: "app/views/login.html",
    controller: "LoginController"
})

.when("/onboarding", {
    templateUrl: "app/views/onboarding.html",
    controller: "OnboardingController"
})

.when("/dashboard", {
    templateUrl: "app/views/dashboard.html",
    controller: "DashboardController"
})

.otherwise({
    redirectTo: "/login"
});

});