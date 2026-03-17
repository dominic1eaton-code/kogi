import { Component, inject } from '@angular/core';
import { Router } from '@angular/router';

@Component({
  selector: 'app-registration',
  imports: [],
  templateUrl: './registration.component.html',
  styleUrl: './registration.component.css',
})
export class RegistrationComponent {
  // Inject the Router service
  private router = inject(Router);

  // Method to handle the navigation
  goToPage(): void {
    // Perform any necessary logic here
    console.log('Navigating to the target page...');

    // Navigate to the specified route
    this.router.navigate(['/onboarding']);
  }
}
