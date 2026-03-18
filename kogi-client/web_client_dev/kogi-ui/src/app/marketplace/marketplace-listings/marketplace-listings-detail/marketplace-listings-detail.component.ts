import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-listings-detail',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-listings-detail.component.html',
  styleUrl: './marketplace-listings-detail.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceListingsDetailComponent {}
