import { Component, inject } from '@angular/core';
import { Router, RouterLink } from '@angular/router';

@Component({
  selector: 'app-registration',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './registration.component.html',
  styleUrl: './registration.component.css',
})
export class RegistrationComponent {
  passwordStrength = 0;

  // Inject the Router service
  private router = inject(Router);

  // Method to handle the navigation
  goToPage(): void {
    // Perform any necessary logic here
    console.log('Navigating to the target page...');

    // Navigate to the specified route
    this.router.navigate(['/onboarding']);
  }

  onPasswordInput(event: Event): void {
    const value = (event.target as HTMLInputElement).value;
    let score = 0;

    if (value.length > 6) {
      score += 25;
    }
    if (value.length > 10) {
      score += 25;
    }
    if (/[A-Z]/.test(value)) {
      score += 25;
    }
    if (/[0-9]/.test(value)) {
      score += 25;
    }

    this.passwordStrength = score;
  }
}
