import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletFundingComponent } from './wallet-funding.component';

describe('WalletFundingComponent', () => {
  let component: WalletFundingComponent;
  let fixture: ComponentFixture<WalletFundingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletFundingComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletFundingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
