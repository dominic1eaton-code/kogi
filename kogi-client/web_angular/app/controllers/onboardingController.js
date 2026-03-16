app.controller("OnboardingController", function($scope, $location){

$scope.selection = "freelancer";

$scope.selectOption = function(option){
    $scope.selection = option;
};

$scope.continue = function(){
    $location.path("/dashboard");
};

});