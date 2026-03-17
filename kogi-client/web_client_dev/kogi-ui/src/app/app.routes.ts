import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { RegistrationComponent } from './registration/registration.component';
import { OnboardingComponent } from './onboarding/onboarding.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { CreatingWorkspaceComponent } from './creating-workspace/creating-workspace.component';
import { PortfolioComponent } from './portfolio/portfolio.component';
import { PortfolioCreateComponent } from './portfolio/portfolio-create/portfolio-create.component';
import { WalletComponent } from './wallet/wallet.component';
import { TestComponent } from './test/test';

export const routes: Routes = [
    {path: '', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
    {path: 'login', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
    {path: 'registration', component: RegistrationComponent, title: 'Registration'},
    {path: 'onboarding', component: OnboardingComponent, title: 'Onboarding'},
    {path: 'creating-workspace', component: CreatingWorkspaceComponent, title: 'Creating Workspace'},
    {path: 'dashboard', component: DashboardComponent, title: 'Dashboard'},
    {path: 'portfolio/new', component: PortfolioCreateComponent, title: 'Create Portfolio'},
    {path: 'portfolio', component: PortfolioComponent, title: 'Portfolio'},
    {path: 'wallet', component: WalletComponent, title: 'Wallet'},
    {path: 'test', component: TestComponent, title: "Testing Page"},
    {path: '**', redirectTo: '' }
];
