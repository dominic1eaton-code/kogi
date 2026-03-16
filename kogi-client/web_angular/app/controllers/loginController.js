app.controller("LoginController", function($scope, $location){

$scope.user = {};

$scope.login = function(){

    console.log("Logging in:", $scope.user);

    // simulate auth success
    $location.path("/onboarding");

};

});