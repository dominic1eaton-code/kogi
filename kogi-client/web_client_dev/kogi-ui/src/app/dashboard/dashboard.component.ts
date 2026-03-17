import { Component, inject } from '@angular/core';
import { Router } from '@angular/router'

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class DashboardComponent {
  

  // Inject the Router service
  private router = inject(Router);

  // Method to handle the navigation
  logout(): void {
    // Perform any necessary logic here
    console.log('Logging out of client session...');

    // Navigate to the specified route
    this.router.navigate(['/login']);
  }
}
