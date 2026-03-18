import { Component, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { LoginComponent } from './index/login/login.component';
import { RegistrationComponent } from './index/registration/registration.component'
import { OnboardingComponent } from './index/onboarding/onboarding.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { TestComponent} from './index/test/test'

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, LoginComponent, RegistrationComponent, OnboardingComponent, DashboardComponent, TestComponent],
  templateUrl: './app.html',
  styleUrl: './app.css'
})

export class App {
  protected readonly title = signal('kogi-ui');
}