import { Component, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive, RouterOutlet  } from '@angular/router'

@Component({
  selector: 'app-office',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './office.component.html',
  styleUrl: './office.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class OfficeComponent {
  

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
