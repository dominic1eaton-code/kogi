import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-listings-mine',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-listings-mine.component.html',
  styleUrl: './marketplace-listings-mine.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceListingsMineComponent {}
