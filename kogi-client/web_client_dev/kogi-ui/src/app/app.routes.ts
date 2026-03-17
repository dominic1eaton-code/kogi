import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { RegistrationComponent } from './registration/registration.component';
import { OnboardingComponent } from './onboarding/onboarding.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { TestComponent } from './test/test';

export const routes: Routes = [
    {path: '', component: LoginComponent, title: "Login Page"},
    {path: 'login', component: LoginComponent, title: "Login Page"},
    {path: 'registration', component: RegistrationComponent, title: 'Registration Page'},
    {path: 'onboarding', component: OnboardingComponent, title: 'Onboarding Page'},
    {path: 'dashboard', component: DashboardComponent, title: 'Dashboard Page'},
    {path: 'test', component: TestComponent, title: "Testing Page"},
    {path: '**', redirectTo: '' }
];
