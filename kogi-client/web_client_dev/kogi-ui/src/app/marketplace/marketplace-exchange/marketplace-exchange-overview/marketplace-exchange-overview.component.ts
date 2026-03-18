import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-overview',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-overview.component.html',
  styleUrl: './marketplace-exchange-overview.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeOverviewComponent {}
