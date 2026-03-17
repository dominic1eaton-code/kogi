import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { RegistrationComponent } from './registration/registration.component';

export const routes: Routes = [
    {path: '', component: LoginComponent, title: "Login Page"},
    {path: 'login', component: LoginComponent, title: "Login Page"},
    {path: 'registration', component: RegistrationComponent, title: 'Registration Page'},
    {path: '**', redirectTo: '' }
];
