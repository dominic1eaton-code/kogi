import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-bids',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-bids.component.html',
  styleUrl: './marketplace-exchange-bids.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeBidsComponent {}
