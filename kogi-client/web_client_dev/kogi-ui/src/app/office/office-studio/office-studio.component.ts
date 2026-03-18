import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-office-studio',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './office-studio.component.html',
  styleUrl: './office-studio.component.css',
  host: {
    class: 'block w-full'
  }
})
export class OfficeStudioComponent {}
