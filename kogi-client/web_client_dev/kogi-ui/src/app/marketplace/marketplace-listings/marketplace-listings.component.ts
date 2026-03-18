import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-listings',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-listings.component.html',
  styleUrl: './marketplace-listings.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceListingsComponent {}
