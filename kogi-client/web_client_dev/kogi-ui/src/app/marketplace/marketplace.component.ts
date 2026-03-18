import { Component } from '@angular/core';
import { NavigationComponent } from '../navigation/navigation.component';

@Component({
  selector: 'app-marketplace',
  standalone: true,
  imports: [NavigationComponent],
  templateUrl: './marketplace.component.html',
  styleUrl: './marketplace.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class MarketplaceComponent {}
