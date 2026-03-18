import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-listings-catalog',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-listings-catalog.component.html',
  styleUrl: './marketplace-listings-catalog.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceListingsCatalogComponent {}
